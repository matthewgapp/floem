use std::{hash::Hash, rc::Rc, vec::IntoIter};

use floem::{
    peniko::Color,
    reactive::{
        create_rw_signal, create_signal, ReadSignal, RwSignal, Scope, SignalGet,
        SignalGetUntracked, SignalUpdate,
    },
    style::{FlexDirection, Style},
    view::View,
    views::{
        label, tree_builder::TreeView, tree_simple, Decorators, Label, ListData, NeverIterate,
        TreeData, TreeNode, TreeProps,
    },
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

#[derive(Clone, Copy)]
struct MyTreeData {
    data: RwSignal<i32>,
    children: ReadSignal<Vec<MyTreeData>>,
}

impl MyTreeData {
    fn leaf(scope: Scope, data: i32) -> Self {
        let data = create_rw_signal(scope, data);
        Self {
            data,
            children: create_rw_signal(scope, vec![]).read_only(),
        }
    }

    fn node(scope: Scope, data: i32, children: Vec<MyTreeData>) -> Self {
        let data = create_rw_signal(scope, data);
        Self {
            data,
            children: create_rw_signal(scope, children).read_only(),
        }
    }
}

impl TreeNode<MyTreeData, Label, Vec<MyTreeData>> for MyTreeData {
    type Item = MyTreeData;
    type K = i32;
    type KeyFn = Box<dyn Fn(&Self::Item) -> Self::K>;
    type Children = ReadSignal<Vec<MyTreeData>>;
    type ViewFn = Box<dyn Fn(&Self::Item) -> Label>;

    fn has_children(&self) -> bool {
        self.children.get_untracked().len() > 0
    }

    fn key_fn(&self) -> Self::KeyFn {
        Box::new(|x: &MyTreeData| {
            println!("key function ran");
            let key = x.data.get();
            println!("key {}", key);
            key
        })
    }

    fn view_fn(&self) -> Self::ViewFn {
        Box::new(|x: &MyTreeData| {
            // panic!("view fn");
            println!("view function ran");
            let x = *x;
            floem::views::label(move || x.data.get().to_string())
        })
    }

    fn children(&self) -> Self::Children {
        println!("children got");
        self.children
    }

    fn node(&self) -> Self::Item {
        MyTreeData {
            data: self.data,
            children: self.children,
        }
    }
}

fn build_tree<T, V, S, I>(tree_node: S) -> TreeView<S::Item, V>
where
    S: TreeNode<T, V, I, Item = S> + Copy + 'static,
    V: View + 'static,
    I: IntoIterator<Item = S> + 'static,
    T: 'static,
{
    use floem::views::tree_builder::{tree_view, Children, Node};
    println!("build tree");

    let parent = Node::new(tree_node.view_fn());
    let children = move || {
        Some(Children::new(
            move || tree_node.children().get(),
            tree_node.key_fn(),
            move |x| {
                println!("inside view function");
                if tree_node.has_children() {
                    build_tree(x)
                } else {
                    let parent = Node::new(tree_node.view_fn());
                    tree_view::<_, _, NeverIterate<_>, S::K>(x, parent, || None)
                }
            },
        ))
    };

    tree_view(tree_node.node(), parent, children)
        .style(|| Style::BASE.flex_direction(FlexDirection::Column))
}

// fn app_view_with_tree_builder() -> impl View {
//     let cx = ViewContext::get_current();
//     use floem::views::tree_builder::{tree_view, Children, Node};

//     let signal = create_rw_signal(cx.scope, {
//         (0..10)
//             .map(|x| create_rw_signal(cx.scope, x))
//             .collect::<Vec<_>>()
//     });

//     let write_signal = signal;

//     // needs a way to tell me to build children
//     // need to get a type that is is a closure that returns IntoIter<T> where T: SignalGet
//     // need a type that returns a closure that is |value: T|  -> V where V: View, T: SignalGet

//     let parent = Node::new(|data: &ReadSignal<_>| {
//         let data = *data;
//         label(move || format!("Parent: {}", data.get()))
//     });
//     let children = Children::new(
//         move || {
//             println!("rebuilding list");
//             let list = write_signal
//                 .get()
//                 .iter()
//                 .map(|x| x.read_only())
//                 .collect::<Vec<_>>();
//             println!("list len: {}", list.len());
//             list
//         },
//         |x: &ReadSignal<_>| x.get(),
//         {
//             Box::new(|x| {
//                 let parent_2 = Node::new(|x: &ReadSignal<_>| {
//                     let x = *x;
//                     label(move || format!("Parent Level 2: {}", x.get()))
//                 });
//                 tree_view::<_, _, NeverIterate<_>, i32>(x, parent_2, || None)
//             })
//         },
//     );

//     let data = create_rw_signal(cx.scope, 0);

//     floem::views::stack(|| {
//         (
//             label(|| "Click me".to_string())
//                 .on_click(move |_| {
//                     write_signal.get_untracked().iter().for_each(|x| {
//                         x.update(|x| {
//                             *x += 1;
//                         })
//                     });
//                     write_signal.update(|x| x.push(create_rw_signal(cx.scope, 0)));
//                     true
//                 })
//                 .keyboard_navigatable()
//                 .focus_visible_style(|| Style::BASE.border(2.).border_color(Color::BLUE))
//                 .style(|| Style::BASE.border(1.0).border_radius(10.0).padding_px(10.0)),
//             // button
//             tree_view(data.read_only(), parent, Some(children))
//                 .style(|| Style::BASE.flex_direction(FlexDirection::Column)),
//         )
//     })
// }

fn app_view_with_tree_node() -> impl View {
    let cx = ViewContext::get_current();
    let scope = cx.scope;
    let tree_data_2 = MyTreeData::node(
        scope,
        1,
        vec![
            MyTreeData::node(scope, 2, vec![]),
            MyTreeData::node(
                scope,
                3,
                vec![
                    MyTreeData::leaf(scope, 5),
                    MyTreeData::node(
                        scope,
                        6,
                        vec![
                            MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            // MyTreeData::node(scope, 7, vec![]),
                            MyTreeData::node(
                                scope,
                                8,
                                vec![
                                    MyTreeData::node(scope, 9, vec![]),
                                    MyTreeData::node(
                                        scope,
                                        10,
                                        vec![
                                            MyTreeData::node(scope, 11, vec![]),
                                            MyTreeData::node(
                                                scope,
                                                12,
                                                vec![
                                                    MyTreeData::node(scope, 13, vec![]),
                                                    MyTreeData::node(
                                                        scope,
                                                        14,
                                                        vec![
                                                            MyTreeData::node(scope, 15, vec![]),
                                                            MyTreeData::node(scope, 16, vec![]),
                                                        ],
                                                    ),
                                                ],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
            MyTreeData::node(scope, 4, vec![]),
        ],
    );

    build_tree(tree_data_2)
}

fn main() {
    floem::launch(app_view_with_tree_node);
}
