use floem::{
    peniko::Color,
    reactive::{create_signal, SignalGet, SignalUpdate},
    style::Style,
    view::View,
    views::{label, stack, tree, Decorators, ParentData},
    ViewContext,
};

fn app_view() -> impl View {
    let cx = ViewContext::get_current();

    tree(floem::views::TreeData::Parent(ParentData { 
        child_view_fn: Box::new(|item| {
            stack()
                .child(label(item))
    }))
}

fn main() {
    floem::launch(app_view);
}
