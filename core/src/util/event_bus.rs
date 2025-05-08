use crossbeam::channel::Receiver;
use std::sync::Arc;
use winit::event_loop::EventLoopProxy;

#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    WorldRequestRedraw,
    Shutdown,
}

pub trait EventProxyTrait<T: 'static + std::fmt::Debug> {
    fn send_event(&self, event: T) -> Result<(), winit::event_loop::EventLoopClosed<T>>;
}

pub struct EventProxy<T: 'static + std::fmt::Debug> {
    event_loop_proxy: Arc<EventLoopProxy<T>>,
}

impl<T: 'static + std::fmt::Debug> EventProxy<T> {
    pub fn new(event_loop_proxy: Arc<EventLoopProxy<T>>) -> Self {
        Self { event_loop_proxy }
    }
}

impl<T: 'static + std::fmt::Debug> EventProxyTrait<T> for EventProxy<T> {
    fn send_event(&self, event: T) -> Result<(), winit::event_loop::EventLoopClosed<T>> {
        self.event_loop_proxy.send_event(event)
    }
}
pub struct EventBusProxy<T: 'static + std::fmt::Debug + Send> {
    receiver: Arc<Receiver<T>>,
    event_loop_proxy: Arc<dyn EventProxyTrait<T> + Send + Sync>,
}

impl<T: 'static + std::fmt::Debug + Send> EventBusProxy<T> {
    pub fn new(
        receiver: &Arc<Receiver<T>>,
        event_loop_proxy: Arc<dyn EventProxyTrait<T> + Send + Sync>,
    ) -> Self {
        Self {
            receiver: receiver.clone(),
            event_loop_proxy,
        }
    }
    pub fn run_tokio(self) {
        tokio::spawn(async move {
            self.start().await;
        });
    }
    async fn start(&self) {
        while let Ok(event) = self.receiver.recv() {
            if let Err(e) = self.event_loop_proxy.send_event(event) {
                eprintln!("Failed to send event: {:?}", e);
            }
        }
    }
}
