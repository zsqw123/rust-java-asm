use crate::impls::apk_load::read_apk;
use crate::impls::util::schedule_task;
use crate::server::OpenFileError;
use crate::ui::{AppContainer, DirInfo, Left};
use crate::{AccessorEnum, AccessorMut, AsmServer, ServerMut};
use log::info;
use rfd::MessageDialogResult::No;
use std::io::{Read, Seek};
use std::ops::DerefMut;
use std::time::Instant;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use zip::ZipArchive;

pub enum ServerMessage {
    Progress(ProgressMessage),
}

pub struct ProgressMessage {
    // 0.0 - 1.0
    pub progress: f32,
    pub in_loading: bool,
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
        let (sender, receiver) = mpsc::channel::<ServerMessage>(5);
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
                        server_ref.on_progress_update(&render_target);
                    }
                }
            }
        });
        sender
    }

    pub(crate) async fn read_apk(
        apk_content: impl Read + Seek,
        sender: Sender<ServerMessage>,
        accessor: AccessorMut,
    ) -> Result<(), OpenFileError> {
        let zip = ZipArchive::new(apk_content)
            .map_err(OpenFileError::LoadZip)?;
        let apk_accessor = read_apk(zip, sender).await?;
        // safe unwrap, no other places in current thread will access it.
        *accessor.lock() = Some(AccessorEnum::Apk(apk_accessor));
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

    fn on_progress_update(&self, render_target: &AppContainer) {
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
