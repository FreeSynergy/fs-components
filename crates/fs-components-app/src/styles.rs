//! The style sheet of the shell.
//!
//! Pattern: Template — only theme variables, so a theme can change everything. Sizes a surface must
//! know in pixels are constants, because the shell tells the surface its size.

/// Height of the title bar in pixels.
pub(crate) const BAR_HEIGHT: f32 = 44.0;
/// Height of the status line in pixels. It is always there, so a message moves nothing.
pub(crate) const STATUS_HEIGHT: f32 = 28.0;
/// Width of the navigation in pixels.
pub(crate) const NAV_WIDTH: f32 = 200.0;

/// Width of an open menu in pixels; a longer label wraps.
pub(crate) const MENU_WIDTH: f32 = 220.0;
/// Height of a line of a menu in pixels, for keeping a panel inside the window.
pub(crate) const MENU_LINE_HEIGHT: f32 = 36.0;
/// Text size of the menus in pixels.
pub(crate) const MENU_TEXT: f32 = 14.0;
/// Room between the title bar's menu and its panel in pixels.
pub(crate) const MENU_GAP: f32 = 4.0;

/// The smallest window in logical pixels: room for bar, status line and navigation.
pub(crate) const MIN_WINDOW: (u32, u32) = (360, 240);

/// CSS of the shell.
pub(crate) const STYLES: &str = r"
.fs-app-root { display: flex; flex-direction: column; background: var(--background); color: var(--text-primary); font-family: var(--font-sans); }
.fs-app-bar { display: flex; flex-direction: row; align-items: center; flex-shrink: 0; overflow: hidden; padding-left: var(--space-3); padding-right: var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--surface); }
.fs-app-title-box { display: flex; flex-direction: row; flex-shrink: 1; min-width: 0; overflow: hidden; }
.fs-app-title { font-weight: 600; }
.fs-app-menus { display: flex; flex-direction: row; align-items: center; flex-shrink: 0; gap: var(--space-1); margin-left: var(--space-3); }
.fs-app-row { display: flex; flex-direction: row; flex-grow: 1; min-height: 0; }
.fs-app-nav { display: flex; flex-direction: column; flex-shrink: 0; gap: var(--space-1); padding: var(--space-2); border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-app-main { display: flex; flex-direction: column; min-width: 0; }
.fs-app-status-line { flex-shrink: 0; overflow: hidden; }
.fs-app-status { display: flex; flex-direction: row; align-items: center; overflow: hidden; padding-left: var(--space-3); padding-right: var(--space-3); border-top-width: 1px; border-top-color: var(--border); color: var(--text-secondary); font-size: var(--text-sm); }
.fs-app-status-problem { background: var(--error-bg); color: var(--text-primary); }
.fs-app-empty { padding: var(--space-6); color: var(--text-secondary); }
";
