use gl;
use 
sdl2::{
    self,
    event::Event,
    keyboard::Keycode,
    video::GLProfile,
};

struct COLOR {
    R: f32,
    G: f32,
    B: f32
}

fn main() {
    let (WIDTH, HEIGHT) = (800, 600);
    
    let SDL = sdl2::init().unwrap();
    let SDL_VIDEO = SDL.video().unwrap();

    let gl_attr = SDL_VIDEO.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    
    let win = SDL_VIDEO.window("Window", WIDTH, HEIGHT).opengl().borderless().build().unwrap(); let GL =
    win.gl_create_context().unwrap(); gl::load_with(|name| SDL_VIDEO.gl_get_proc_address(name) as *const _);
    
    let mut event_pump = SDL.event_pump().unwrap();

    println!( "Start" );

    let mut C = COLOR{ R: 0., G: 0., B: 0.};

    'running: loop { 
        for event in event_pump.poll_iter() {
            println!( "{:#?}", event );
            match event { 
                Event::Quit{ .. } | Event::KeyDown{ keycode: Some( Keycode::Escape ), .. } => {
                    break 'running; 
                },
                Event::MouseMotion{ x, y, .. } => { 
                    C.R = x as f32 / WIDTH as f32; 
                    C.G = y as f32 / HEIGHT as f32;
                },
                _  => {},
            }
        }
        
        unsafe { 
            gl::ClearColor( C.R, C.G, C.B, 1. );
            gl::Clear( gl::COLOR_BUFFER_BIT );
        }

        win.gl_swap_window();
    }

    println!( "End" );
}
