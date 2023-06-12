use crate::view::View;

use super::{
    scroll, virtual_list, VirtualList, VirtualListDirection, VirtualListItemSize, VirtualListVector,
};
use std::hash::Hash;

// a tree is a list of tree items
// a tree item can either return a tree or a leaf

// a tree is a a view made up of indented lists
enum TreeData<T, IF, I, K, KF, VF, V, L>
where
    T: 'static,
    IF: Fn() -> I + 'static,
    I: VirtualListVector<T>,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + 'static,
    VF: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    // TODO: fix generics here
    Leaf(LeafData<T, VF, L>),
    Branch(BranchData<T, IF, KF, VF>),
}

struct LeafData<T, VF, L>
where
    T: 'static,
    VF: Fn(T) -> L + 'static,
    L: View,
{
    item: T,
    view_fn: Box<VF>,
}

struct BranchData<T, IF, KF, VF> {
    item_size: VirtualListItemSize<T>,
    each_fn: Box<IF>,
    key_fn: Box<KF>,
    view_fn: Box<VF>,
}

struct Tree<T, VF, V, L>
where
    T: 'static,
    VF: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    child: TreeChild<T, VF, V, L>,
}

enum TreeChild<T, VF, V, L>
where
    T: 'static,
    VF: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    Leaf(L),
    Branch(VirtualList<V, VF, T>),
}

pub fn tree<T, IF, I, KF, K, VF, V, L>(
    tree_data: TreeData<T, IF, I, K, KF, VF, V, L>,
) -> Tree<T, VF, V, L>
where
    T: 'static,
    IF: Fn() -> I + 'static,
    I: VirtualListVector<T>,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + 'static,
    VF: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    // let cx = ViewContext::get_current();
    // let long_list: im::Vector<i32> = (0..100).collect();
    // let (long_list, set_long_list) = create_signal(cx.scope, long_list);
    Tree {
        child: match tree_data {
            TreeData::Branch(branch) => TreeChild::Branch(virtual_list(
                VirtualListDirection::Vertical,
                VirtualListItemSize::Fixed(Box::new(|| 32.0)),
                branch.each_fn,
                branch.key_fn,
                *branch.view_fn,
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
