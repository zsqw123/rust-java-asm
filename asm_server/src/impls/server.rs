use crate::ui::AppContainer;
use crate::AsmServer;
use log::info;
use std::ops::DerefMut;
use std::time::Instant;

pub enum ServerMessage {
    Progress(ProgressMessage),
}

pub struct ProgressMessage {
    // 0.0 - 1.0
    pub progress: f32,
    pub in_loading: bool,
}

pub struct FileOpenContext {
    pub path: String,
    pub start_time: Instant,
}


impl AsmServer {
    pub fn on_file_opened(
        &self,
        context: &FileOpenContext,
        render_target: AppContainer,
    ) {
        let FileOpenContext { path, start_time } = context;
        info!("open file {path} cost: {:?}", start_time.elapsed());
        self.render_to_app(render_target);
    }

    pub fn on_progress_update(&self, render_target: &AppContainer) {
        let current_loading_state = &self.loading_state;
        let mut top = render_target.top().lock();
        let top_mut = top.deref_mut();
        (*top_mut).loading_state = current_loading_state.clone();
    }
}
