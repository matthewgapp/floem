use crate::view::View;

use super::{
    scroll, virtual_list, VirtualList, VirtualListDirection, VirtualListItemSize, VirtualListVector,
};
use std::hash::Hash;

// a tree is a list of tree items
// a tree item can either return a tree or a leaf

// a tree is a a view made up of indented lists
enum TreeData<T, IF, KF, PVF, CVF, L>
where
    CVF: Fn(T) -> L + 'static,
{
    Leaf(LeafData<T, CVF>),
    Parent(ParentData<T, IF, KF, PVF>),
}

struct LeafData<T, CVF> {
    item: T,
    view_fn: Box<CVF>,
}

struct ParentData<T, IF, KF, PVF> {
    parent_view_fn: Box<PVF>,

    item_size: VirtualListItemSize<T>,
    each_fn: Box<IF>,
    key_fn: Box<KF>,
    child_view_fn: Box<PVF>,
}

struct Tree<T, CFV, V, L>
where
    T: 'static,
    CFV: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    child: TreeChild<T, CFV, V, L>,
}

enum TreeChild<T, CVF, V, L>
where
    T: 'static,
    CVF: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    Leaf(L),
    Branch(VirtualList<V, CVF, T>),
}

fn tree<T, IF, I, KF, K, PVF, CVF, V, L>(
    tree_data: TreeData<T, IF, KF, PVF, CVF, L>,
) -> Tree<T, PVF, V, L>
where
    K: Hash + Eq + 'static,
    I: VirtualListVector<T> + 'static,
    IF: Fn() -> I + 'static,
    KF: Fn(&T) -> K + 'static,
    PVF: Fn(T) -> V + 'static,
    CVF: Fn(T) -> L + 'static,
    T: 'static,
    V: View,
    L: View,
{
    // let cx = ViewContext::get_current();
    // let long_list: im::Vector<i32> = (0..100).collect();
    // let (long_list, set_long_list) = create_signal(cx.scope, long_list);
    Tree {
        child: match tree_data {
            TreeData::Parent(branch) => TreeChild::Branch(virtual_list(
                VirtualListDirection::Vertical,
                VirtualListItemSize::Fixed(Box::new(|| 32.0)),
                branch.each_fn,
                branch.key_fn,
                *branch.child_view_fn,
            )),
            TreeData::Leaf(leaf) => TreeChild::Leaf((leaf.view_fn)(leaf.item)),
        },
    }
    // let (selected, set_selected) = create_signal(cx.scope, 0);
    // let list_width = 180.0;
    // let item_height = 32.0;
    // scroll(move || {
    //     virtual_list(
    //         VirtualListDirection::Vertical,
    //         item_size,
    //         VirtualListItemSize::Fixed(Box::new(|| 32.0)),
    //         move || long_list.get(),
    //         move |item| *item,
    //         move |item| {
    //             let index = long_list
    //                 .get_untracked()
    //                 .iter()
    //                 .position(|it| *it == item)
    //                 .unwrap();
    //             let (is_checked, set_is_checked) = create_signal(cx.scope, true);
    //             container(move || {
    //                 stack(move || {
    //                     (
    //                         checkbox(is_checked).on_click(move |_| {
    //                             set_is_checked.update(|checked: &mut bool| *checked = !*checked);
    //                             true
    //                         }),
    //                         label(move || item.to_string())
    //                             .style(|| Style::BASE.height_px(32.0).font_size(32.0)),
    //                         container(move || {
    //                             label(move || " X ".to_string())
    //                                 .on_click(move |_| {
    //                                     print!("Item Removed");
    //                                     set_long_list.update(|x| {
    //                                         x.remove(index);
    //                                     });
    //                                     true
    //                                 })
    //                                 .style(|| {
    //                                     Style::BASE
    //                                         .height_px(18.0)
    //                                         .font_weight(Weight::BOLD)
    //                                         .color(Color::RED)
    //                                         .border(1.0)
    //                                         .border_color(Color::RED)
    //                                         .border_radius(16.0)
    //                                         .margin_right_px(5.0)
    //                                 })
    //                                 .hover_style(|| {
    //                                     Style::BASE.color(Color::WHITE).background(Color::RED)
    //                                 })
    //                         })
    //                         .style(|| {
    //                             Style::BASE
    //                                 .flex_basis(Dimension::Points(0.0))
    //                                 .flex_grow(1.0)
    //                                 .justify_content(Some(JustifyContent::FlexEnd))
    //                         }),
    //                     )
    //                 })
    //                 .style(move || {
    //                     Style::BASE
    //                         .height_px(item_height)
    //                         .width_px(list_width)
    //                         .items_center()
    //                 })
    //             })
    //             .on_click(move |_| {
    //                 set_selected.update(|v: &mut usize| {
    //                     *v = long_list.get().iter().position(|it| *it == item).unwrap();
    //                 });
    //                 true
    //             })
    //         },
    //     )
    // })
}
