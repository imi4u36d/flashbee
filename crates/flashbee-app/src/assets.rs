use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "icons/archive.svg" => Some(include_bytes!("../assets/icons/archive.svg")),
            "icons/arrow-down.svg" => Some(include_bytes!("../assets/icons/arrow-down.svg")),
            "icons/arrow-down-to-line.svg" => {
                Some(include_bytes!("../assets/icons/arrow-down-to-line.svg"))
            }
            "icons/arrow-left.svg" => Some(include_bytes!("../assets/icons/arrow-left.svg")),
            "icons/arrow-up.svg" => Some(include_bytes!("../assets/icons/arrow-up.svg")),
            "icons/arrow-up-from-line.svg" => {
                Some(include_bytes!("../assets/icons/arrow-up-from-line.svg"))
            }
            "icons/check.svg" => Some(include_bytes!("../assets/icons/check.svg")),
            "icons/check-circle-2.svg" => {
                Some(include_bytes!("../assets/icons/check-circle-2.svg"))
            }
            "icons/chevron-down.svg" => Some(include_bytes!("../assets/icons/chevron-down.svg")),
            "icons/chevron-right.svg" => Some(include_bytes!("../assets/icons/chevron-right.svg")),
            "icons/circle-dot.svg" => Some(include_bytes!("../assets/icons/circle-dot.svg")),
            "icons/clock-3.svg" => Some(include_bytes!("../assets/icons/clock-3.svg")),
            "icons/cloud-cog.svg" => Some(include_bytes!("../assets/icons/cloud-cog.svg")),
            "icons/cloud-download.svg" => {
                Some(include_bytes!("../assets/icons/cloud-download.svg"))
            }
            "icons/code-2.svg" => Some(include_bytes!("../assets/icons/code-2.svg")),
            "icons/code-xml.svg" => Some(include_bytes!("../assets/icons/code-xml.svg")),
            "icons/columns-2.svg" => Some(include_bytes!("../assets/icons/columns-2.svg")),
            "icons/command.svg" => Some(include_bytes!("../assets/icons/command.svg")),
            "icons/copy.svg" => Some(include_bytes!("../assets/icons/copy.svg")),
            "icons/corner-down-left.svg" => {
                Some(include_bytes!("../assets/icons/corner-down-left.svg"))
            }
            "icons/download.svg" => Some(include_bytes!("../assets/icons/download.svg")),
            "icons/ellipsis.svg" => Some(include_bytes!("../assets/icons/ellipsis.svg")),
            "icons/external-link.svg" => Some(include_bytes!("../assets/icons/external-link.svg")),
            "icons/eye.svg" => Some(include_bytes!("../assets/icons/eye.svg")),
            "icons/eye-off.svg" => Some(include_bytes!("../assets/icons/eye-off.svg")),
            "icons/file-code-2.svg" => Some(include_bytes!("../assets/icons/file-code-2.svg")),
            "icons/filter.svg" => Some(include_bytes!("../assets/icons/filter.svg")),
            "icons/folder-git-2.svg" => Some(include_bytes!("../assets/icons/folder-git-2.svg")),
            "icons/folder-open.svg" => Some(include_bytes!("../assets/icons/folder-open.svg")),
            "icons/folder-plus.svg" => Some(include_bytes!("../assets/icons/folder-plus.svg")),
            "icons/git-branch.svg" => Some(include_bytes!("../assets/icons/git-branch.svg")),
            "icons/git-commit-horizontal.svg" => {
                Some(include_bytes!("../assets/icons/git-commit-horizontal.svg"))
            }
            "icons/git-fork.svg" => Some(include_bytes!("../assets/icons/git-fork.svg")),
            "icons/git-pull-request.svg" => {
                Some(include_bytes!("../assets/icons/git-pull-request.svg"))
            }
            "icons/history.svg" => Some(include_bytes!("../assets/icons/history.svg")),
            "icons/inbox.svg" => Some(include_bytes!("../assets/icons/inbox.svg")),
            "icons/layers.svg" => Some(include_bytes!("../assets/icons/layers.svg")),
            "icons/list.svg" => Some(include_bytes!("../assets/icons/list.svg")),
            "icons/list-filter.svg" => Some(include_bytes!("../assets/icons/list-filter.svg")),
            "icons/loader-circle.svg" => Some(include_bytes!("../assets/icons/loader-circle.svg")),
            "icons/minus.svg" => Some(include_bytes!("../assets/icons/minus.svg")),
            "icons/panel-left.svg" => Some(include_bytes!("../assets/icons/panel-left.svg")),
            "icons/panel-left-close.svg" => {
                Some(include_bytes!("../assets/icons/panel-left-close.svg"))
            }
            "icons/pencil.svg" => Some(include_bytes!("../assets/icons/pencil.svg")),
            "icons/plus.svg" => Some(include_bytes!("../assets/icons/plus.svg")),
            "icons/refresh-cw.svg" => Some(include_bytes!("../assets/icons/refresh-cw.svg")),
            "icons/rotate-ccw.svg" => Some(include_bytes!("../assets/icons/rotate-ccw.svg")),
            "icons/rows-3.svg" => Some(include_bytes!("../assets/icons/rows-3.svg")),
            "icons/scan-search.svg" => Some(include_bytes!("../assets/icons/scan-search.svg")),
            "icons/search.svg" => Some(include_bytes!("../assets/icons/search.svg")),
            "icons/settings-2.svg" => Some(include_bytes!("../assets/icons/settings-2.svg")),
            "icons/shield-check.svg" => Some(include_bytes!("../assets/icons/shield-check.svg")),
            "icons/sparkles.svg" => Some(include_bytes!("../assets/icons/sparkles.svg")),
            "icons/square-pen.svg" => Some(include_bytes!("../assets/icons/square-pen.svg")),
            "icons/text-search.svg" => Some(include_bytes!("../assets/icons/text-search.svg")),
            "icons/trash-2.svg" => Some(include_bytes!("../assets/icons/trash-2.svg")),
            "icons/triangle-alert.svg" => {
                Some(include_bytes!("../assets/icons/triangle-alert.svg"))
            }
            "icons/upload.svg" => Some(include_bytes!("../assets/icons/upload.svg")),
            "icons/x.svg" => Some(include_bytes!("../assets/icons/x.svg")),
            _ => None,
        };
        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
