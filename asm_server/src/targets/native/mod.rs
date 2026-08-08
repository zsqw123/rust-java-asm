mod runtime;

pub(crate) use runtime::{file_handle_path, reveal_parent, schedule_task, Instant};

use crate::impls::apk_load::ApkAccessor;
use crate::impls::server::ServerMessage;
use crate::server::OpenFileError;
use std::io::{Read, Seek};
use tokio::sync::mpsc::Sender;
use zip::ZipArchive;

pub(crate) async fn read_apk(
    zip_archive: ZipArchive<impl Read + Seek>, sender: Sender<ServerMessage>,
) -> Result<ApkAccessor, OpenFileError> {
    crate::impls::apk_load::read_apk(zip_archive, sender, || async {}).await
}
