mod language_servers;

use zed::lsp::{Completion, Symbol};
use zed::settings::LspSettings;
use zed::{serde_json, CodeLabel, LanguageServerId};
use zed_extension_api::{self as zed, Result};

use crate::language_servers::svls;
use zed_extension_api as zed;

struct VerilogExtension {
    svls: Option<Svls>,
}

impl zed::Extension for VerilogExtension {
    fn new() -> Self {
        Self { svls: None }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        match language_server_id.as_ref() {
            Svls::LANGUAGE_SERVER_ID => {
                let svls = self.svls.get_or_insert_with(|| Svls::new());

                Ok(zed::Command {
                    command: svls.server_script_path(worktree)?,
                    args: vec!["stdio".into()],
                    env: worktree.shell_env(),
                })
            }
            language_server_id => Err(format!("unknown language server: {language_server_id}")),
        }
    }

    fn label_for_symbol(
        &self,
        language_server_id: &LanguageServerId,
        symbol: Symbol,
    ) -> Option<CodeLabel> {
        match language_server_id.as_ref() {
            Svls::LANGUAGE_SERVER_ID => self.svls.as_ref()?.label_for_symbol(symbol)
            _ => None
        }
    }

    fn label_for_completion(
        &self,
        language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        match language_server_id.as_ref() {
            Svls::LANGUAGE_SERVER_ID => {
                self.svls.as_ref()?.label_for_completion(completion)
            }
            _ => None,
        }
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let initialization_options =
            LspSettings::for_worktree(language_server_id.as_ref(), worktree)
                .ok()
                .and_then(|lsp_settings| lsp_settings.initialization_options.clone())
                .unwrap_or_default();

        Ok(Some(serde_json::json!(initialization_options)))
    }
}

zed::register_extension!(VerilogExtension);
