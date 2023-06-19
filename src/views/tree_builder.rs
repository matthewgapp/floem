use leptos_reactive::create_effect;
use taffy::style::LengthPercentageAuto;

use crate::{
    id::Id,
    style::Style,
    view::{ChangeFlags, View},
    views::{label, Label},
    ViewContext,
};
use std::{hash::Hash, marker::PhantomData, rc::Rc};

use super::{
    empty,
    list_type_erased::{list, List},
    Decorators,
};

pub struct Node<T, V> {
    // need to type erase, otherwise every node will be tied to the same concrete closure
    view_fn: Box<dyn Fn(&T) -> V>,
}

impl<T, V> Node<T, V> {
    pub fn new(view_fn: impl Fn(&T) -> V + 'static) -> Self {
        Self {
            view_fn: Box::new(view_fn),
        }
    }
}

pub struct Children<T, V, I, K>
where
    T: 'static,
    V: View + 'static,
{
    iter_fn: Box<dyn Fn() -> I>,
    key_fn: Box<dyn Fn(&T) -> K>,
    view_fn: Box<dyn Fn(T) -> TreeView<T, V>>,
}

impl<T, V, I, K> Children<T, V, I, K>
where
    V: View,
{
    pub fn new(
        iter_fn: impl Fn() -> I + 'static,
        key_fn: impl Fn(&T) -> K + 'static,
        view_fn: impl Fn(T) -> TreeView<T, V> + 'static,
    ) -> Self {
        Self {
            iter_fn: Box::new(iter_fn),
            key_fn: Box::new(key_fn),
            view_fn: Box::new(view_fn),
        }
    }
}

pub struct TreeView<T, V>
where
    T: 'static,
    V: View + 'static,
{
    id: Id,
    cx: ViewContext,
    parent: V,
    // children_data: Option<Children<T, V, I, K>>,
    children: Option<Box<List<TreeView<T, V>, T>>>,
    // children: Option<Box<Label>>,
    phantom: PhantomData<T>,
}

pub fn tree_view<T, V, I, K>(
    value: T,
    parent: Node<T, V>,
    children: impl Fn() -> Option<Children<T, V, I, K>> + 'static,
) -> TreeView<T, V>
where
    K: Hash + Eq + 'static,
    I: IntoIterator<Item = T> + 'static,
    T: 'static,
    V: View + 'static,
{
    let (id, (parent, children)) = ViewContext::new_id_with_child(|| {
        let parent = (parent.view_fn)(&value);

        let cx = ViewContext::get_current();

        create_effect(cx.scope, move |_| {
            println!("create effect ran");
            // TODO: think that this is built with the wrong context or something
            let list: Option<Box<dyn FnOnce() -> Box<_>>> = children().map(|c| {
                // let cx = ViewContext::get_current();
                // Box::new(label(|| "hi".to_string()))
                println!(
                    "creating list within context id of {:?}",
                    ViewContext::get_current().id
                );
                Box::new(|| {
                    Box::new(list(c.iter_fn, c.key_fn, c.view_fn)).style(|| {
                        Style::BASE
                            .flex_direction(crate::style::FlexDirection::Column)
                            .margin_left(LengthPercentageAuto::Points(20.))
                    })
                }) as Box<dyn FnOnce() -> Box<List<_, _>>>
            });

            println!("update state called in tree view to id {:?}", cx.id);
            // cx.id.update_state(vec![1, 2, 3], false);
            cx.id.update_state(list, false);
            // cx.id.update_state("Second version".to_string(), false);
        });

        // let children = Some(Box::new(label(|| "First version".to_string())));
        // let children = Some(Box::new(label(|| "First version".to_string())));
        (parent, ())
    });
    println!("tree view created");

    let cx = ViewContext::get_current();
    let mut child_cx = cx;
    child_cx.id = id;
    println!("child context id {:?}", id);

    TreeView {
        cx: child_cx,
        id,
        parent,
        // children_data: None,
        children: None,
        phantom: PhantomData,
    }
}

impl<T, V> View for TreeView<T, V>
where
    T: 'static,
    V: View + 'static,
{
    fn id(&self) -> Id {
        self.id
    }

    fn child(&mut self, id: Id) -> Option<&mut dyn View> {
        println!("child called with id {:?}", id);
        if self.parent.id() == id {
            return Some(&mut self.parent);
        } else if let Some(list) = &mut self.children {
            if list.id() == id {
                return Some(list);
            }
        }
        println!("no child found");
        return None;
    }

    fn children(&mut self) -> Vec<&mut dyn View> {
        let mut children = vec![&mut self.parent as &mut dyn View];
        if let Some(list) = &mut self.children {
            children.push(list);
        }
        children
    }

    fn update(
        &mut self,
        cx: &mut crate::context::UpdateCx,
        state: Box<dyn std::any::Any>,
    ) -> crate::view::ChangeFlags {
        println!(
            "------update function in tree builder with id {:?}------",
            self.id
        );
        // println!("state type id {:?}", state);
        // if let Ok(string) = state.downcast() {
        //     let message: String = *string;
        //     println!("message in update state {}", message);
        // }
        if let Ok(state) = state.downcast() {
            println!("state captured in downcast");
            // let string: String = *state;

            ViewContext::save();
            ViewContext::set_current(self.cx);
            println!("current context set to {:?}", self.cx.id);
            let create_list: Option<Box<dyn FnOnce() -> Box<List<TreeView<T, V>, T>>>> = *state;
            // let children: Option<Children<>>  = *state;
            self.children = create_list.map(|list| list());
            // self.children = Some(Box::new(label(move || format!("message is: {}", string))));
            // let numbers = Rc::new(state);

            // self.children = Some(
            //     Box::new(list(
            //         move || (1..3),
            //         |x| *x,
            //         Box::new(|x: i32| label(move || x.to_string())),
            //     ))
            //     .style(|| {
            //         Style::BASE
            //             .flex_direction(crate::style::FlexDirection::Column)
            //             .margin_left(LengthPercentageAuto::Points(20.))
            //     }),
            // );
            ViewContext::restore();
            println!("children set and layout requested");
            cx.request_layout(self.id());
            ChangeFlags::LAYOUT
        } else {
            ChangeFlags::empty()
        }
        // ChangeFlags::empty()
    }

    fn event(
        &mut self,
        cx: &mut crate::context::EventCx,
        id_path: Option<&[Id]>,
        event: crate::event::Event,
    ) -> bool {
        false
    }

    fn paint(&mut self, cx: &mut crate::context::PaintCx) {
        self.parent.paint_main(cx);
        if let Some(children) = self.children.as_mut() {
            println!("painting children");
            children.paint_main(cx);
        }
    }

    fn layout(&mut self, cx: &mut crate::context::LayoutCx) -> taffy::prelude::Node {
        cx.layout_node(self.id, true, |cx| {
            let mut nodes = vec![self.parent.layout_main(cx)];
            if let Some(children) = self.children.as_mut() {
                nodes.push(children.layout_main(cx))
            }
            println!("layout nodes: {:?}", nodes);
            nodes
        })
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "TreeBuilderView".into()
    }
}
