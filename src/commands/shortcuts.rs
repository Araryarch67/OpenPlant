//! Keyboard shortcut reference sheet.

/// Shortcut reference (Help > Keyboard Shortcuts), draw.io layout.
pub const SHORTCUTS: &[(&str, &[(&str, &str)])] = &[
    (
        "File",
        &[
            ("Ctrl+N", "New"),
            ("Ctrl+O", "Open"),
            ("Ctrl+S", "Save"),
            ("Ctrl+Shift+S", "Save as"),
            ("Ctrl+Q", "Quit"),
        ],
    ),
    (
        "Edit",
        &[
            ("Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y", "Undo / Redo"),
            ("Ctrl+X / Ctrl+C / Ctrl+V", "Cut / Copy / Paste"),
            ("Ctrl+D / Ctrl+Enter", "Duplicate"),
            ("Delete / Backspace", "Delete"),
            ("Ctrl+Delete", "Delete with connections"),
            ("F2 / Enter / Double-click", "Edit label"),
            ("Alt+Shift+F / Alt+Shift+V", "Copy / Paste size"),
            ("Ctrl+Shift+Y", "Autosize"),
        ],
    ),
    (
        "Selection",
        &[
            ("Ctrl+A", "Select all"),
            ("Ctrl+Shift+I / Ctrl+Shift+E", "Select vertices / edges"),
            ("Ctrl+Shift+A / Esc", "Select none"),
            ("Tab / Shift+Tab", "Select next / previous"),
            ("Shift+Click", "Toggle selection"),
            ("Drag on blank / Alt+Drag", "Rubber band"),
        ],
    ),
    (
        "Arrange",
        &[
            ("Arrow / Shift+Arrow", "Move 1px / grid"),
            ("Ctrl+Arrow / Ctrl+Shift+Arrow", "Resize 1px / grid"),
            ("Ctrl+Drag", "Copy while moving"),
            ("Shift+Drag", "Constrain to axis"),
            ("Alt+Drag", "Move without grid snap"),
            ("Ctrl+Shift+F / Ctrl+Shift+B", "To front / back"),
            ("Ctrl+Alt+Shift+F / B", "Forward / backward"),
        ],
    ),
    (
        "View",
        &[
            ("Ctrl+Wheel / Alt+Wheel", "Zoom"),
            ("Wheel / Shift+Wheel", "Scroll / horizontal scroll"),
            ("Space+Drag / Middle / Right-drag", "Pan"),
            ("Ctrl++ / Ctrl+-", "Zoom in / out"),
            ("Home / Ctrl+0", "Reset view"),
            ("Ctrl+Shift+H / Ctrl+J", "Fit window"),
            ("Ctrl+Shift+G", "Grid"),
            ("Ctrl+Shift+O", "Outline"),
            ("Ctrl+Shift+K / Ctrl+Shift+P", "Sidebar / Navigator"),
            ("F1", "Command palette"),
        ],
    ),
];
