use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

#[derive(Debug)]
pub struct BoxState {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}


#[derive(Debug)]
pub struct State {
    pub window: Window,
    pub user_state: UserState,
    pub box_state: BoxState,
}

#[derive(Debug)]
pub struct UserState {
    pub pos_x: i32,
    pub pos_y: i32,
}

impl State {
    pub async fn new(event_loop: &EventLoop<()>) -> State {
        let window = WindowBuilder::new()
            .with_title("Rendering a Box with Pixels")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .build(event_loop)
            .unwrap();

        Self {
            window,
            user_state: UserState { pos_x: 0, pos_y: 0 },
            box_state: BoxState {
                box_x: 24,
                box_y: 16,
                velocity_x: 1,
                velocity_y: 1,
            },
        }
    }

    pub fn redraw(&self) {
        self.window.request_redraw();
    }

    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.inner_size()
    }
}

impl BoxState {
    pub fn move_box(&mut self) {
        self.box_x += 1;
        self.box_y += 1;
    }
}

impl UserState {
    pub fn default() -> Self {
        Self {
            pos_x: 0,
            pos_y: 0,
        }
    }

    pub fn new(x: i32, y: i32) -> Self {
        Self {
            pos_x: x,
            pos_y: y
        }
    }

    pub fn move_user(&mut self, x: Option<i32>, y: Option<i32>) {
        if x.is_some() {
            self.pos_x += x.unwrap();
        }

        if y.is_some() {
            self.pos_y += y.unwrap();
        }
    }
}