use glazier::{kurbo::Size, WindowBuilder};
use leptos_reactive::{create_runtime, raw_scope_and_disposer, Scope, SignalGetUntracked};

use crate::{
    app_handle::AppHandle, id::Id, view::View, views::debug_view::debug_view, window::WindowConfig,
};

type AppEventCallback = dyn Fn(&AppEvent);

pub fn launch<V: View + 'static>(app_view: impl Fn() -> V + 'static) {
    Application::new()
        .window(app_view, None)
        // .window(app_view, Some(WindowConfig::default().debug()))
        .run()
}

pub enum AppEvent {
    WillTerminate,
}

/// Floem top level application
/// This is the entry point of the application.
pub struct Application {
    application: glazier::Application,
    scope: Scope,
    event_listener: Option<Box<AppEventCallback>>,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl glazier::AppHandler for Application {
    fn command(&mut self, _id: u32) {}

    fn will_terminate(&mut self) {
        if let Some(action) = self.event_listener.as_ref() {
            action(&AppEvent::WillTerminate);
        }
    }
}

impl Application {
    pub fn new() -> Self {
        let runtime = create_runtime();
        let (scope, _) = raw_scope_and_disposer(runtime);
        Self {
            scope,
            application: glazier::Application::new().unwrap(),
            event_listener: None,
        }
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }

    pub fn on_event(mut self, action: impl Fn(&AppEvent) + 'static) -> Self {
        self.event_listener = Some(Box::new(action));
        self
    }

    fn standard_window_builder(&self, config: Option<&WindowConfig>) -> WindowBuilder {
        let mut builder = WindowBuilder::new(self.application.clone()).size(
            config
                .as_ref()
                .and_then(|c| c.size)
                .unwrap_or_else(|| Size::new(800.0, 600.0)),
        );
        if let Some(position) = config.as_ref().and_then(|c| c.position) {
            builder = builder.position(position);
        }
        if let Some(show_titlebar) = config.as_ref().and_then(|c| c.show_titlebar) {
            builder = builder.show_titlebar(show_titlebar);
        }
        builder
    }

    /// create a new window for the application, if you want multiple windows,
    /// just chain more window method to the builder
    pub fn window<V: View + 'static>(
        self,
        app_view: impl FnOnce() -> V + 'static,
        config: Option<WindowConfig>,
    ) -> Self {
        let builder = self.standard_window_builder(config.as_ref());

        let debug_window = self.standard_window_builder(None).title("Debug");
        let _ = self.scope.child_scope(move |cx| {
            let is_debug = config.as_ref().map(|c| c.debug).unwrap_or(false);

            if is_debug {
                let mut app = AppHandle::new(cx, app_view);
                let debug_state = app.init_debug_state();
                let debug_window = debug_window
                    .handler(Box::new(AppHandle::new(cx, || {
                        debug_view::<_, _, Id>(debug_state.root_tree_node().get_untracked())
                    })))
                    .build()
                    .unwrap();
                let builder = builder.handler(Box::new(app));
                let window = builder.build().unwrap();
                window.show();
                debug_window.show();
            } else {
                let app = AppHandle::new(cx, app_view);
                let builder = builder.handler(Box::new(app));
                let window = builder.build().unwrap();
                window.show();
            }
        });
        self
    }

    pub fn run(self) {
        let application = self.application.clone();
        application.run(Some(Box::new(self)));
    }
}
