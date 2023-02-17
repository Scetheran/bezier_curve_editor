
#[derive(Clone, Copy, Debug)]
pub enum ApplicationEvent {
    WindowResized { width: u32, height: u32 },
    MouseMoved { x: u32, y: u32 },
    MouseLeftButtonPressed,
    MouseLeftButtonReleased,
    MouseRightButtonPressed,
    MouseRightButtonReleased,
    FramebufferResized { width: u32, height: u32},
}