use crate::targets::{schedule_task, Instant};
use crate::server::OpenFileError;
use crate::ui::{AppContainer, DirInfo, Left, ToastKind};
use crate::{AccessorEnum, AccessorMut, AsmServer, ServerMut};
use log::info;
use std::ops::DerefMut;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub enum ServerMessage {
    Progress(ProgressMessage),
    Error(String),
}

pub struct ProgressMessage {
    // 0.0 - 1.0
    pub progress: f32,
    pub in_loading: bool,
    pub message: String,
}

pub struct FileOpenContext {
    pub file_name: String,
    pub start_time: Instant,
}


impl AsmServer {
    pub(crate) fn create_message_handler(
        server: &ServerMut, render_target: &AppContainer,
    ) -> Sender<ServerMessage> {
        let server = server.clone();
        let render_target = render_target.clone();
        let (sender, receiver) = mpsc::channel::<ServerMessage>(50);
        let mut receiver = receiver;
        schedule_task(async move {
            while let Some(msg) = receiver.recv().await {
                let mut server = server.lock();
                let server_ref = server.deref_mut();
                let Some(server_ref) = server_ref else { continue };
                match msg {
                    ServerMessage::Progress(progress) => {
                        server_ref.loading_state.loading_progress = progress.progress;
                        server_ref.loading_state.in_loading = progress.in_loading;
                        server_ref.loading_state.loading_message = progress.message;
                        server_ref.on_progress_update(&render_target);
                    }
                    ServerMessage::Error(message) => {
                        server_ref.loading_state.in_loading = false;
                        server_ref.loading_state.loading_message = "Load failed".to_owned();
                        server_ref.on_progress_update(&render_target);
                        render_target.push_toast(ToastKind::Error, message);
                    }
                }
            }
        });
        sender
    }

    pub(crate) async fn read_files(
        inputs: Vec<(String, Vec<u8>)>, sender: Sender<ServerMessage>, accessor: AccessorMut,
    ) -> Result<(), OpenFileError> {
        if inputs.is_empty() {
            return Err(OpenFileError::Custom("no input files selected".to_owned()));
        }
        // Keep file-type dispatch here so Jar/Class accessors can be added without
        // making the generic server entry point depend on DEX loading details.
        let dex_accessor = crate::targets::read_dex_inputs(inputs, sender).await?;
        // safe unwrap, no other places in current thread will access it.
        *accessor.lock() = Some(AccessorEnum::Dex(dex_accessor));
        Ok(())
    }

    pub(crate) fn on_file_opened(
        &self,
        context: &FileOpenContext,
        render_target: AppContainer,
    ) {
        let FileOpenContext { file_name: path, start_time } = context;
        info!("open file {path} cost: {:?}", start_time.elapsed());
        self.render_to_app(render_target);
    }

    pub(crate) fn on_progress_update(&self, render_target: &AppContainer) {
        let current_loading_state = &self.loading_state;
        let mut top = render_target.top().lock();
        let top_mut = top.deref_mut();
        (*top_mut).loading_state = current_loading_state.clone();
    }

    fn render_to_app(&self, app: AppContainer) {
        let classes = self.read_classes();
        let start = Instant::now();
        let dir_info = DirInfo::from_classes(&classes);
        info!("resolve dir info cost: {:?}", start.elapsed());
        app.set_left(Left { root_node: dir_info, offset_key: None, hint_key: None });
    }
}

#[cfg(test)]
mod tests {
    use super::{AsmServer, ServerMessage};
    use crate::{Accessor, AccessorEnum};
    use crate::server::OpenFileError;
    use parking_lot::Mutex;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    #[test]
    fn read_files_rejects_empty_input() {
        let (sender, _receiver) = mpsc::channel(1);
        let accessor = Arc::new(Mutex::new(None));
        let error = futures::executor::block_on(AsmServer::read_files(
            Vec::new(), sender, accessor,
        )).unwrap_err();
        assert!(matches!(error, OpenFileError::Custom(message)
            if message == "no input files selected"));
    }

    #[test]
    fn read_single_dex() {
        let (sender, mut receiver) = mpsc::channel(16);
        let accessor = Arc::new(Mutex::new(None));
        let bytes = include_bytes!("../../../asm/tests/res/dex/classes14.dex").to_vec();

        futures::executor::block_on(AsmServer::read_files(
            vec![("classes14.dex".to_owned(), bytes)], sender, accessor.clone(),
        )).unwrap();

        let messages: Vec<_> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
        for message in &messages {
            if let ServerMessage::Progress(progress) = message {
                assert!(progress.progress == 0.0 || progress.progress >= 0.9);
            }
        }
        let Some(ServerMessage::Progress(progress)) = messages.last() else {
            panic!("single DEX did not report completion");
        };
        assert_eq!(progress.progress, 1.0);
        assert!(!progress.in_loading);

        let accessor = accessor.lock();
        let Some(AccessorEnum::Dex(accessor)) = accessor.as_ref() else {
            panic!("single DEX was not loaded");
        };
        assert!(!accessor.read_classes().is_empty());
        assert!(accessor.peek_source("classes14.dex").is_some());
    }

    #[test]
    fn read_multiple_dex_files() {
        let (sender, _receiver) = mpsc::channel(16);
        let accessor = Arc::new(Mutex::new(None));
        let bytes = include_bytes!("../../../asm/tests/res/dex/classes14.dex");

        futures::executor::block_on(AsmServer::read_files(
            vec![
                ("first.dex".to_owned(), bytes.to_vec()),
                ("second.dex".to_owned(), bytes.to_vec()),
            ],
            sender,
            accessor.clone(),
        )).unwrap();

        let accessor = accessor.lock();
        let Some(AccessorEnum::Dex(accessor)) = accessor.as_ref() else {
            panic!("multiple DEX files were not loaded");
        };
        assert!(accessor.peek_source("first.dex").is_some());
        assert!(accessor.peek_source("second.dex").is_some());
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn read_single_apk_prefixes_entry_names() {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let dex = include_bytes!("../../../asm/tests/res/dex/classes14.dex");
        writer.start_file("classes14.dex", SimpleFileOptions::default()).unwrap();
        writer.write_all(dex).unwrap();
        let apk = writer.finish().unwrap().into_inner();
        let (sender, _receiver) = mpsc::channel(16);
        let accessor = Arc::new(Mutex::new(None));

        futures::executor::block_on(AsmServer::read_files(
            vec![("sample.apk".to_owned(), apk)], sender, accessor.clone(),
        )).unwrap();

        let accessor = accessor.lock();
        let Some(AccessorEnum::Dex(accessor)) = accessor.as_ref() else {
            panic!("single APK was not loaded");
        };
        assert!(accessor.peek_source("sample.apk!classes14.dex").is_some());
        assert!(accessor.peek_source("classes14.dex").is_none());
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn read_apks_nested_apk() {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        fn zip_file(name: &str, bytes: &[u8]) -> Vec<u8> {
            let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
            writer.start_file(name, SimpleFileOptions::default()).unwrap();
            writer.write_all(bytes).unwrap();
            writer.finish().unwrap().into_inner()
        }

        let dex = include_bytes!("../../../asm/tests/res/dex/classes14.dex");
        let apk = zip_file("classes14.dex", dex);
        let apks = zip_file("base.apk", &apk);
        let (sender, _receiver) = mpsc::channel(16);
        let accessor = Arc::new(Mutex::new(None));

        futures::executor::block_on(AsmServer::read_files(
            vec![("sample.apks".to_owned(), apks)], sender, accessor.clone(),
        )).unwrap();

        let accessor = accessor.lock();
        let Some(AccessorEnum::Dex(accessor)) = accessor.as_ref() else {
            panic!("nested APK was not loaded");
        };
        let source = accessor
            .peek_source("sample.apks!base.apk!classes14.dex")
            .expect("nested DEX source was not found");
        assert_eq!(source.exportable_name.as_ref(), "classes14.dex");
    }
}
