//! Code-related display components.

pub mod agent;
pub mod ansi_parser;
pub mod api_endpoint;
pub mod artifact;
pub mod commit;
pub mod env_variables;
pub mod file_tree;
pub mod jsx_preview;
pub mod package_info;
pub mod sandbox;
pub mod schema_display;
pub mod snippet;
pub mod stack_trace;
pub mod terminal;
pub mod test_results;
pub mod web_preview;

pub use agent::{
    AgentCard, AgentContent, AgentHeader, AgentInstructions, AgentOutput, AgentTool, AgentToolDef,
    AgentTools, AgentView,
};
pub use ansi_parser::{AnsiSpan, AnsiStyle, parse_ansi, parse_ansi_with_style};
pub use api_endpoint::{
    ApiEndpointView, EndpointParameter, EndpointProperty, HttpMethod, ParameterLocation,
};
pub use artifact::{
    Artifact, ArtifactAction, ArtifactActions, ArtifactClose, ArtifactContent, ArtifactDescription,
    ArtifactHeader, ArtifactTitle,
};
pub use commit::{
    // Top-level
    Commit,
    // Actions
    CommitActions,
    // Header subcomponents
    CommitAuthor,
    CommitAuthorAvatar,
    CommitContent,
    CommitCopyButton,
    CommitFileAdditions,
    CommitFileChanges,
    CommitFileData,
    CommitFileDeletions,
    CommitFileIcon,
    CommitFileInfo,
    CommitFilePath,
    // File display subcomponents
    CommitFileRow,
    CommitFileStatus,
    CommitFiles,
    CommitHash,
    // Structural subcomponents
    CommitHeader,
    CommitInfo,
    CommitMessage,
    CommitMetadata,
    CommitSeparator,
    CommitTimestamp,
    // Data types
    FileStatus,
    author_initials,
    // Helpers
    format_relative_time,
};
pub use env_variables::{CopyFormat, EnvVar, EnvironmentVariablesView};
pub use file_tree::{FileTreeView, TreeNode};
pub use jsx_preview::{
    JsxPreview, JsxPreviewContent, JsxPreviewError, JsxPreviewErrorDisplay, JsxPreviewHeader,
    close_unclosed_tags,
};
pub use package_info::{
    ChangeType, Dependency, PackageInfoChangeType, PackageInfoContent, PackageInfoDependencies,
    PackageInfoDependency, PackageInfoDescription, PackageInfoHeader, PackageInfoName,
    PackageInfoVersion, PackageInfoView,
};
pub use sandbox::{SandboxStatus, SandboxTab, SandboxView, status_badge as sandbox_status_badge};
pub use schema_display::SchemaDisplayView;
pub use snippet::Snippet;
pub use stack_trace::{
    ParsedStackTrace, StackFrame, StackTraceActions, StackTraceContent, StackTraceError,
    StackTraceErrorMessage, StackTraceErrorType, StackTraceFrames, StackTraceHeader,
    StackTraceView, parse_stack_trace,
};
pub use terminal::{
    TerminalActions, TerminalClearButton, TerminalContent, TerminalHeader, TerminalStatus,
    TerminalTitle, TerminalView,
};
pub use test_results::{
    Test, TestCase, TestError, TestResults, TestResultsHeader, TestResultsProgress,
    TestResultsView, TestStatus, TestSuite, TestSummary,
};
pub use web_preview::{
    ConsoleEntry, ConsoleLevel, WebPreview, WebPreviewBody, WebPreviewConsole,
    WebPreviewNavigation, WebPreviewNavigationButton, WebPreviewUrl,
};
