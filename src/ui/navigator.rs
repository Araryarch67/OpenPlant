//! Model explorer and editors column.

use crate::app::*;
use crate::model::*;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn navigator(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let q = self.explorer_search.read(cx).value().to_string();
        let mut rows = vec![];
        self.tree_rows(&self.wb.project.root, 0, &q, &mut rows, cx);
        let editors = self.property_editor(window, cx);
        div()
            .relative()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(self.wb.navigator_w))
            .h_full()
            .bg(c(PANEL))
            .child(header("MODEL EXPLORER"))
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .px_3()
                    .pb(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(field(&self.explorer_search).cleanable(true)),
                    )
                    .child(
                        button_sm("sort-az", "A-Z")
                            .when(self.explorer_sorted, |d| {
                                d.border_color(c(BLUE)).text_color(c(BLUE))
                            })
                            .tooltip(|w, cx| Tooltip::new("Sort A-Z").build(w, cx))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.explorer_sorted = !this.explorer_sorted;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .id("tree")
                    .test_support()
                    .flex_1()
                    .max_h(px(300.))
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_1()
                    .pb_2()
                    .children(rows),
            )
            .child(header("EDITORS"))
            .child(
                div()
                    .id("editors")
                    .test_support()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_3()
                    .pb_3()
                    .child(editors),
            )
            .child(self.resizer(false, cx))
            .into_any_element()
    }

    pub(crate) fn tree_rows(
        &self,
        node: &ModelNode,
        depth: usize,
        q: &str,
        out: &mut Vec<AnyElement>,
        cx: &mut Context<Self>,
    ) {
        let ql = q.to_lowercase();
        // A node stays visible when it or any descendant matches.
        fn matches(n: &ModelNode, ql: &str) -> bool {
            n.name.to_lowercase().contains(ql) || n.children.iter().any(|c| matches(c, ql))
        }
        if !q.is_empty() && !matches(node, &ql) {
            return;
        }
        let label = match node.kind {
            NodeKind::Project => self.wb.project.name.clone(),
            NodeKind::Diagram => format!("{} - Model", node.name),
            _ => node.name.clone(),
        }
        .replace('\n', " ");
        let collapsed = self.collapsed.contains(&node.id);
        let has_kids = !node.children.is_empty();
        let selected = self.wb.selected_node.as_deref() == Some(node.id.as_str());
        let (a, b) = (node.id.clone(), node.id.clone());
        let is_diagram = node.kind == NodeKind::Diagram;
        out.push(
            div()
                .id(eid(format!("tree-{}", node.id)))
                .test_support()
                .flex()
                .items_center()
                .gap(px(6.))
                .rounded(px(6.))
                .py(px(3.))
                .pr_2()
                .pl(px(8.0 + depth as f32 * 14.0))
                .whitespace_nowrap()
                .cursor_pointer()
                .hover(|s| s.bg(c(HOVER)))
                .when(selected, |d| d.bg(sel_bg()))
                .child(
                    div()
                        .id(eid(format!("twist-{}", node.id)))
                        .test_support()
                        .w(px(12.))
                        .flex()
                        .flex_none()
                        .justify_center()
                        .when(has_kids, |d| {
                            d.child(chevron(if collapsed { "right" } else { "down" }, c(MUTED)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    if !this.collapsed.remove(&b) {
                                        this.collapsed.insert(b.clone());
                                    }
                                    cx.notify();
                                }))
                        }),
                )
                .child(kind_icon(node.kind))
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    if is_diagram {
                        this.wb.activate_diagram(&a);
                    }
                    this.wb.select_node(Some(a.clone()));
                    cx.notify();
                }))
                .into_any_element(),
        );
        if collapsed {
            return;
        }
        let mut kids: Vec<&ModelNode> = node.children.iter().collect();
        if self.explorer_sorted {
            kids.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        for k in kids {
            self.tree_rows(k, depth + 1, q, out, cx);
        }
    }
}
