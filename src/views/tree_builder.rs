use leptos_reactive::create_effect;
use taffy::style::LengthPercentageAuto;

use crate::{
    id::Id,
    style::Style,
    view::{ChangeFlags, View},
    views::label,
    ViewContext,
};
use std::hash::Hash;

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
    parent: V,
    children: Option<Box<List<TreeView<T, V>, T>>>,
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
    let (id, parent) = ViewContext::new_id_with_child(|| {
        let parent = (parent.view_fn)(&value);

        let cx = ViewContext::get_current();

        create_effect(cx.scope, move |_| {
            println!("create effect ran");
            let list = children().map(|c| {
                // label(|| "hi".to_string())
                Box::new(list(c.iter_fn, c.key_fn, c.view_fn)).style(|| {
                    Style::BASE
                        .flex_direction(crate::style::FlexDirection::Column)
                        .margin_left(LengthPercentageAuto::Points(20.))
                })
            });

            cx.id.update_state(list, false);
        });

        parent
    });
    println!("tree view created");

    TreeView {
        id,
        parent,
        children: None,
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
        if self.parent.id() == id {
            return Some(&mut self.parent);
        } else if let Some(list) = &mut self.children {
            if list.id() == id {
                return Some(list);
            }
        }
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
        if let Ok(state) = state.downcast() {
            self.children = *state;
            cx.request_layout(self.id());
            ChangeFlags::LAYOUT
        } else {
            ChangeFlags::empty()
        }
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
            children.paint_main(cx);
        }
    }

    fn layout(&mut self, cx: &mut crate::context::LayoutCx) -> taffy::prelude::Node {
        cx.layout_node(self.id, true, |cx| {
            let mut nodes = vec![self.parent.layout_main(cx)];
            if let Some(children) = self.children.as_mut() {
                nodes.push(children.layout_main(cx))
            }
            nodes
        })
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "TreeBuilderView".into()
    }
}
