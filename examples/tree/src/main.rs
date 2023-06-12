use std::rc::Rc;

use floem::{
    peniko::Color,
    reactive::{create_signal, SignalGet, SignalUpdate},
    style::{FlexDirection, Style},
    view::View,
    views::{label, stack, tree_simple, Decorators, ListData, TreeProps},
    ViewContext,
};

/*



Learnings:
1. Recursive nature means that you need recursive elements to be boxed.
2. Closure types (concrete) cannot call themselves. If a given generic param is both recursive, meaning that the



 */

fn app_view() -> impl View {
    let cx = ViewContext::get_current();
    let long_list: im::Vector<i32> = (0..10).collect();
    let (long_list, _set_long_list) = create_signal(cx.scope, long_list);

    // let view_fn = Box::new(|item| label(|| format!("Child: {}", item)));
    let view_fn = Rc::new(Box::new(|text: i32| label(move || text.to_string())));

    tree_simple(TreeProps {
        data: 0,
        parent_view_fn: view_fn.clone(),
        children: Some(ListData {
            each_fn: Box::new(move || long_list.get()),
            key_fn: Box::new(|x: &i32| *x),
            view_fn: Box::new(|x| {
                Box::new(label(move || format!("Child: {}", x))) as Box<dyn View>
            }),
        }),
    })
    .style(|| Style::BASE.flex_direction(FlexDirection::Column))
}

fn main() {
    floem::launch(app_view);
}
