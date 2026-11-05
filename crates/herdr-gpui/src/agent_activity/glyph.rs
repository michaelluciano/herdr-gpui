//! The small drawing a Claude agent's row shows for what it is doing, and the
//! one Claude Code's tool names pick while it works.

/// One drawn status or tool mark. Each is a one-colour svg that takes its
/// colour from the row, so a state never relies on hue alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Glyph {
    /// Working with no tool to name, and any tool without a mark of its own.
    Hammer,
    Brain,
    Sleep,
    Subagents,
    Waiting,
    Blocked,
    Terminal,
    Read,
    Edit,
    Write,
    Notebook,
    Grep,
    Glob,
    WebFetch,
    WebSearch,
    Todo,
    Skill,
    Plan,
    Plug,
    Ask,
}

impl Glyph {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 20] = [
        Self::Hammer,
        Self::Brain,
        Self::Sleep,
        Self::Subagents,
        Self::Waiting,
        Self::Blocked,
        Self::Terminal,
        Self::Read,
        Self::Edit,
        Self::Write,
        Self::Notebook,
        Self::Grep,
        Self::Glob,
        Self::WebFetch,
        Self::WebSearch,
        Self::Todo,
        Self::Skill,
        Self::Plan,
        Self::Plug,
        Self::Ask,
    ];

    /// The embedded asset that draws it.
    pub(crate) fn path(self) -> &'static str {
        match self {
            Self::Hammer => "icons/status-hammer.svg",
            Self::Brain => "icons/status-brain.svg",
            Self::Sleep => "icons/status-sleep.svg",
            Self::Subagents => "icons/status-subagents.svg",
            Self::Waiting => "icons/status-waiting.svg",
            Self::Blocked => "icons/status-blocked.svg",
            Self::Terminal => "icons/tool-bash.svg",
            Self::Read => "icons/tool-read.svg",
            Self::Edit => "icons/tool-edit.svg",
            Self::Write => "icons/tool-write.svg",
            Self::Notebook => "icons/tool-notebook.svg",
            Self::Grep => "icons/tool-grep.svg",
            Self::Glob => "icons/tool-glob.svg",
            Self::WebFetch => "icons/tool-webfetch.svg",
            Self::WebSearch => "icons/tool-websearch.svg",
            Self::Todo => "icons/tool-todo.svg",
            Self::Skill => "icons/tool-skill.svg",
            Self::Plan => "icons/tool-plan.svg",
            Self::Plug => "icons/tool-mcp.svg",
            Self::Ask => "icons/tool-ask.svg",
        }
    }
}

/// The mark for Claude Code's tool `name`: its own for each built-in tool,
/// the plug for any MCP tool, the hammer for the rest.
pub(crate) fn tool_glyph(name: &str) -> Glyph {
    match name {
        "Bash" => Glyph::Terminal,
        "Read" => Glyph::Read,
        "Edit" | "MultiEdit" => Glyph::Edit,
        "Write" => Glyph::Write,
        "NotebookEdit" => Glyph::Notebook,
        "Grep" => Glyph::Grep,
        "Glob" => Glyph::Glob,
        "WebFetch" => Glyph::WebFetch,
        "WebSearch" => Glyph::WebSearch,
        "Task" | "Agent" => Glyph::Subagents,
        "TodoWrite" => Glyph::Todo,
        "Skill" => Glyph::Skill,
        "ExitPlanMode" | "EnterPlanMode" => Glyph::Plan,
        "AskUserQuestion" => Glyph::Ask,
        _ if name.starts_with("mcp__") => Glyph::Plug,
        _ => Glyph::Hammer,
    }
}

/// The longest tool name a row spells beside its mark.
const NAME_LIMIT: usize = 20;

/// The tool name to spell beside its mark: only where the mark alone says
/// little, an MCP tool (`github.create_issue`) or one without a mark.
pub(crate) fn tool_name(name: &str) -> Option<String> {
    let shown = match tool_glyph(name) {
        Glyph::Plug => name.trim_start_matches("mcp__").replacen("__", ".", 1),
        Glyph::Hammer => name.to_owned(),
        _ => return None,
    };
    if shown.is_empty() {
        return None;
    }
    if shown.chars().count() <= NAME_LIMIT {
        return Some(shown);
    }
    let cut: String = shown.chars().take(NAME_LIMIT - 1).collect();
    Some(format!("{cut}…"))
}
