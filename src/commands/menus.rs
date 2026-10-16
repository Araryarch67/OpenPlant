//! Menu bar definition.

pub struct MenuItem {
    pub label: &'static str,
    pub cmd: Option<&'static str>,
    pub kbd: Option<&'static str>,
}

pub struct MenuDef {
    pub label: &'static str,
    pub items: &'static [MenuItem],
}

pub(crate) const fn it(label: &'static str, cmd: &'static str, kbd: &'static str) -> MenuItem {
    MenuItem {
        label,
        cmd: Some(cmd),
        kbd: if kbd.is_empty() { None } else { Some(kbd) },
    }
}

pub(crate) const SEP: MenuItem = MenuItem {
    label: "",
    cmd: None,
    kbd: None,
};

/// Menu bar as in `shots/`; draw.io's Arrange items live
/// under Format so the bar itself stays unchanged.
pub const MENUS: &[MenuDef] = &[
    MenuDef {
        label: "File",
        items: &[
            it("New", "file.new", "Ctrl+N"),
            it("Open...", "file.open", "Ctrl+O"),
            it("Save", "file.save", "Ctrl+S"),
            it("Save As...", "file.save-as", "Ctrl+Shift+S"),
            SEP,
            it("Export PNG...", "file.export-png", ""),
            it("Export SVG...", "file.export-svg", ""),
            it("Export PlantUML...", "file.export-puml", ""),
            it("Import PlantUML...", "file.import-puml", ""),
            SEP,
            it("Quit", "app.quit", "Ctrl+Q"),
        ],
    },
    MenuDef {
        label: "Edit",
        items: &[
            it("Undo", "edit.undo", "Ctrl+Z"),
            it("Redo", "edit.redo", "Ctrl+Shift+Z"),
            SEP,
            it("Cut", "edit.cut", "Ctrl+X"),
            it("Copy", "edit.copy", "Ctrl+C"),
            it("Paste", "edit.paste", "Ctrl+V"),
            it("Duplicate", "edit.duplicate", "Ctrl+D"),
            it("Delete", "edit.delete", "Del"),
            SEP,
            it("Select All", "edit.select-all", "Ctrl+A"),
            it("Select Vertices", "edit.select-vertices", "Ctrl+Shift+I"),
            it("Select Edges", "edit.select-edges", "Ctrl+Shift+E"),
            it("Select None", "edit.select-none", "Ctrl+Shift+A"),
            SEP,
            it("Edit Label", "edit.label", "F2"),
            it("Copy Size", "edit.copy-size", "Alt+Shift+F"),
            it("Paste Size", "edit.paste-size", "Alt+Shift+V"),
        ],
    },
    MenuDef {
        label: "Format",
        items: &[
            it("Autosize", "format.autosize", "Ctrl+Shift+Y"),
            it("Line Solid", "format.line-solid", ""),
            it("Line Dashed", "format.line-dashed", ""),
            it("Line Dotted", "format.line-dotted", ""),
            it("Clear Style", "format.clear", ""),
            SEP,
            it("To Front", "arrange.front", "Ctrl+Shift+F"),
            it("To Back", "arrange.back", "Ctrl+Shift+B"),
            it("Bring Forward", "arrange.forward", "Ctrl+Alt+Shift+F"),
            it("Send Backward", "arrange.backward", "Ctrl+Alt+Shift+B"),
            SEP,
            it("Align Left", "align.left", ""),
            it("Align Center", "align.center", ""),
            it("Align Right", "align.right", ""),
            it("Align Top", "align.top", ""),
            it("Align Middle", "align.middle", ""),
            it("Align Bottom", "align.bottom", ""),
            it("Distribute Horizontally", "align.dist-h", ""),
            it("Distribute Vertically", "align.dist-v", ""),
        ],
    },
    MenuDef {
        label: "Model",
        items: &[
            it("Add Class", "diagram.add-class", ""),
            it("Add Interface", "diagram.add-interface", ""),
            it("Add Package", "diagram.add-package", ""),
        ],
    },
    MenuDef {
        label: "Tools",
        items: &[it("Command Palette...", "view.palette", "F1")],
    },
    MenuDef {
        label: "View",
        items: &[
            it("Sidebar", "view.sidebar", "Ctrl+Shift+K"),
            it("Navigator", "view.navigator", "Ctrl+Shift+P"),
            it("Outline", "view.outline", "Ctrl+Shift+O"),
            it("Grid", "view.grid", "Ctrl+Shift+G"),
            SEP,
            it("Zoom In", "view.zoom-in", "Ctrl++"),
            it("Zoom Out", "view.zoom-out", "Ctrl+-"),
            it("Reset View", "view.reset", "Home"),
            it("Fit Window", "view.fit", "Ctrl+Shift+H"),
        ],
    },
    MenuDef {
        label: "Window",
        items: &[
            it("Minimize", "window.minimize", ""),
            it("Close Window", "app.quit", ""),
        ],
    },
    MenuDef {
        label: "Help",
        items: &[
            it("Keyboard Shortcuts", "help.shortcuts", "Ctrl+/"),
            it("About OpenPlant", "help.about", ""),
        ],
    },
];
