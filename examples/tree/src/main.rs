use floem::{
    peniko::Color,
    reactive::{create_signal, SignalGet, SignalUpdate},
    style::Style,
    view::View,
    views::{label, stack, tree, Decorators, ParentData, TreeData},
    ViewContext,
};

fn app_view() -> impl View {
    let cx = ViewContext::get_current();
    let long_list: im::Vector<i32> = (0..1000000).collect();
    let (long_list, _set_long_list) = create_signal(cx.scope, long_list);

    let view_fn = Box::new(|item| label(|| format!("Child: {}", item)));

    tree(floem::views::TreeData::Parent(ParentData {
        parent_view_fn: view_fn,
        item_size: floem::views::VirtualListItemSize::Fixed(Box::new(|| 20.)),
        each_fn: Box::new(|| long_list.get()),
        key_fn: Box::new(|item| item),
        child_view_fn: view_fn,
    }))
}

fn main() {
    floem::launch(app_view);
}
