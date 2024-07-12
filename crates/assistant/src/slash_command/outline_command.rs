use super::{SlashCommand, SlashCommandOutput};
use anyhow::{anyhow, Context as _, Result};
use assistant_slash_command::ArgumentCompletion;
use editor::Editor;
use gpui::{AppContext, Task, WeakView};
use language::LspAdapterDelegate;
use std::sync::Arc;
use std::{path::Path, sync::atomic::AtomicBool};
use ui::WindowContext;
use workspace::Workspace;

pub(crate) struct OutlineSlashCommand;

impl SlashCommand for OutlineSlashCommand {
    fn name(&self) -> String {
        "outline".into()
    }

    fn description(&self) -> String {
        "insert outline for active tab".into()
    }

    fn menu_text(&self) -> String {
        "Insert Outline for Active Tab".into()
    }

    fn complete_argument(
        self: Arc<Self>,
        _query: String,
        _cancel: Arc<AtomicBool>,
        _workspace: Option<WeakView<Workspace>>,
        _cx: &mut AppContext,
    ) -> Task<Result<Vec<ArgumentCompletion>>> {
        Task::ready(Err(anyhow!("this command does not require argument")))
    }

    fn requires_argument(&self) -> bool {
        false
    }

    fn run(
        self: Arc<Self>,
        _argument: Option<&str>,
        workspace: WeakView<Workspace>,
        _delegate: Arc<dyn LspAdapterDelegate>,
        cx: &mut WindowContext,
    ) -> Task<Result<SlashCommandOutput>> {
        let output = workspace.update(cx, |workspace, cx| {
            let Some(active_item) = workspace.active_item(cx) else {
                return Task::ready(Err(anyhow!("no active tab")));
            };
            let Some(buffer) = active_item
                .downcast::<Editor>()
                .and_then(|editor| editor.read(cx).buffer().read(cx).as_singleton())
            else {
                return Task::ready(Err(anyhow!("active tab is not an editor")));
            };

            let snapshot = buffer.read(cx).snapshot();
            let path = snapshot.resolve_file_path(cx, true);

            cx.background_executor().spawn(async move {
                let outline = snapshot
                    .outline(None)
                    .context("no outline for active tab")?;

                let mut outline_text = format!(
                    "Symbols for {}:\n",
                    path.as_deref().unwrap_or(Path::new("untitled")).display()
                );
                for item in &outline.path_candidates {
                    outline_text.push_str("- ");
                    outline_text.push_str(&item.string);
                    outline_text.push('\n');
                }

                Ok(SlashCommandOutput {
                    text: outline_text,
                    sections: Vec::new(),
                    run_commands_in_text: false,
                })
            })
        });

        output.unwrap_or_else(|error| Task::ready(Err(error)))
    }
}
