use crate::chip::compiler::Opcode;
use crate::chip::screen::renderer::Disposable;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use glutin::ContextBuilder;
use glutin::window::WindowBuilder;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ ControlFlow, EventLoop};

use std::io::Read;
use std::fs::File;

mod cpu;
use cpu::CPU;

mod screen;
use screen::Screen;

mod memory;
use memory::Memory;

mod keys;
use keys::KeyPad;

mod compiler;
pub use compiler::Compiler; 

mod assembler;
pub use assembler::Assemblier;

pub mod utils;


static mut DELTA_TIME: f64 = 0.0;
static mut LAST_TIME:f64 = 0.0;

struct Chip{
    cpu: Box<CPU>, opcode: Box<Opcode>,
    memory: Box<Memory>, screen: Box<Screen>, keys: Box<KeyPad>, loaded: bool
}

impl Chip{
    fn new(screen: Screen)->Self{
        return Chip{
            cpu: Box::new(CPU::new()), opcode:Box::new(Opcode::new(0)), screen: Box::new(screen),
            memory: Box::new(Memory::new()), keys: Box::new(KeyPad::new()), loaded: false
        }
    }

    pub fn reset(&mut self){
        self.cpu.reset();
        self.opcode.clear();
        self.screen.clear();
        self.memory.clear();
        self.keys.reset();
        self.loaded = false;
    }

    pub fn load(&mut self, rom: &str){
        match &mut File::open(rom){
            Ok(file) =>{
                let mut init: Vec<u8> = Vec::new();
                if rom.ends_with(".c8"){
                    if let Err(error) = file.read_to_end(init.as_mut()){
                        panic!("{}", error);
                    }
                }else if rom.ends_with(".asm"){
                    let mut data = String::new();
                    if let Err(error) = file.read_to_string(&mut data){
                        panic!("{}", error);
                    }
                    let mut assembler = Assemblier::new(data.as_ref());
                    init = assembler.run();
                }
                self.memory.load(init.as_ref());
                self.loaded = true;
            },
            Err(error) =>{ panic!("{}", error); }
        }
    }

    pub fn is_loaded(&self)->bool{
        return self.loaded;
    }

    pub fn run(&mut self)-> bool {
        let opcode:Opcode  = Opcode::new(self.cpu.fetch(self.memory.as_ref()));
        match &opcode & 0xf000{
            0x0000 =>{
                match &opcode & 0x00ff{
                    0x00e0 => self.cpu.clear_screen(self.screen.as_mut()), // clear screen
                    0x00ee => self.cpu.ret(), // pop stack pointer
                    _ => { self.cpu.invalid(&opcode); return false; }
                }
            },
            0x1000 => self.cpu.jump(&opcode),      // jump to address in opcode
            0x2000 => self.cpu.call(&opcode),      // jump to address in opcode,
            0x3000 => self.cpu.if_eq(&opcode),      //skip next instruction if vx is equal to kk
            0x4000 => self.cpu.if_not_eq(&opcode),   //skip next instruction if vx is not equal to kk
            0x5000 => self.cpu.if_eq_reg(&opcode),   //skip next instruction if vx is equal to vy
            0x6000 => self.cpu.set(&opcode),       // set vx = kk
            0x7000 => self.cpu.add(&opcode),       // 7XNN - add kk to vx
            0x8000 =>{
                match &opcode & 0x000f{
                    0x0000 => self.cpu.assign(&opcode),    // set vx to vy
                    0x0001 => self.cpu.bit_or(&opcode),     // vx = vx or vy
                    0x0002 => self.cpu.bit_and(&opcode),    // vx = vx and vy
                    0x0003 => self.cpu.bit_xor(&opcode),    // vx = vx xor vy
                    0x0004 => self.cpu.add_reg(&opcode),    // vx = vx + vy, vf = carry
                    0x0005 => self.cpu.sub_reg(&opcode),    // vx = vx-vy vf-not borrow if vx > vy vf = 1 else vf = 0
                    0x0006 => self.cpu.shift_right(&opcode),// SHR vx{, vy}  vx = vx shr 1
                    0x0007 => self.cpu.sub_copy(&opcode),   // vx = vy-vx vf-not borrow if vy > vx vf = 1 else vf = 0
                    0x000e => self.cpu.shift_left(&opcode),
                    _ => { self.cpu.invalid(&opcode); return false; }
                }
            },
            0x9000 => self.cpu.if_not_eq_reg(&opcode),    // skip next instruction if vx not equal to vy
            0xa000 => self.cpu.set_i(&opcode),          // set i = nnn
            0xb000 => self.cpu.jump_v0(&opcode),
            0xc000 => self.cpu.and_rand(&opcode),       // vx = random byte and kk
            0xd000 => self.cpu.draw(&opcode, self.screen.as_mut(), self.memory.as_ref()), //display n-byte sprite starting at memory location I at (vx/vy), set vf=collision
            0xe000 =>{ 
                match &opcode & 0x000f{
                    0x000e => self.cpu.if_key(&opcode, self.keys.as_ref()), // skip new instruction if keys with the value of vx is pressed
                    0x0001 => self.cpu.if_not_key(&opcode, self.keys.as_ref()), // skip new instruction if keys with the value of vx is not pressed
                    _ => { self.cpu.invalid(&opcode); return false; }
                }
            },
            0xf000 =>{
                match &opcode & 0x00ff{
                    0x0007 => self.cpu.get_delay(&opcode), // get vx = delay timer table
                    0x000a => self.cpu.wait_key(&opcode, self.keys.as_mut()),
                    0x0015 => self.cpu.set_delay(&opcode),      // set delay timer = vx,
                    0x0018 => self.cpu.set_sound(&opcode),      // set sound timer = vx
                    0x001e => self.cpu.add_i(&opcode),          // I = I + vx
                    0x0029 => self.cpu.get_font(&opcode),       // set i = location of 5 bit sprite for digit vx
                    0x0030 => self.cpu.get_font16(&opcode),     // set i = location of 10 bit sprite for digit vx    
                    0x0033 => self.cpu.bcd(&opcode, self.memory.as_mut()),           // store BCD representation of vx in memory location I, I + 1, I + 2
                    0x0055 => self.cpu.reg_dump(&opcode, self.memory.as_mut()),       // read register v0 through vx in memory starting at location I
                    0x0065 => self.cpu.reg_load(&opcode, self.memory.as_ref()),       // read register v0 through vx in memory starting at location I
                    //0x0075 => self.cpu.user_reg_dump(&opcode),   // store register v0 through vx in memory starting at location I
                    //0x0085 => self.cpu.user_reg_load(&opcode),   // read register v0 through vx in memory starting at location I
                    _ => { self.cpu.invalid(&opcode); return false; }
                }
            },
            _ => { self.cpu.invalid(&opcode); return false; }
        }
        self.cpu.update();
        return true;
    }
}

pub fn start(file: &str){
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Chip-8 Emulator").with_inner_size(glutin::dpi::LogicalSize::new(800, 480));
    let context = unsafe {
        let context = ContextBuilder::new().build_windowed(window, &event_loop).unwrap();
        context.make_current().unwrap()
    };

    gl::load_with(| symbol | context.get_proc_address(symbol) as *const _);

    let mut chip = Chip::new(Screen::new(800, 480));

    unsafe {
        LAST_TIME = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::FrontFace(gl::CW);
        gl::CullFace(gl::BACK);
        gl::Enable(gl::CULL_FACE);

        chip.reset();
        chip.load(file);
    }

    let mut ok = true;

    event_loop.run(move | event, _, control_flow| {   
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent{ event, ..} => match event{
                WindowEvent::CloseRequested => {
                    unsafe{
                        chip.screen.dispose();
                    }
                    *control_flow = ControlFlow::Exit
                },
                WindowEvent::Resized(size) => {
                    unsafe{ gl::Viewport(0, 0, size.width as i32, size.height as i32); }
                    context.resize(size);
                    chip.screen.resize(size.width, size.height);
                },
                WindowEvent::KeyboardInput { input, .. } =>{
                    println!("{:?}", input);
                },
                __ => {}
            },
            _ =>{}
        }

        unsafe {
            let current = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
            DELTA_TIME = current - LAST_TIME;
            LAST_TIME = current;
            gl::ClearColor(0.0, 0.0, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            if chip.is_loaded() && ok{
                ok = chip.run();
            }
            chip.screen.render();
        }
        context.swap_buffers().unwrap();
    });
}

/*pub fn start(file: &str, debug: bool){
    let mut running = true;
    let mut monitor: Option<PistonWindow> = if debug{
        Option::Some(WindowSettings::new("monitor", [800, 480]).build().unwrap())
    }else{
        Option::None
    };

    let mut main: PistonWindow = WindowSettings::new("Chip-8", [800, 480]).build().unwrap();

    let mut chip = Chip::new(Screen::new(800, 480));
    chip.reset();
    chip.load(file);

    while running{
        if let Some(event) = main.next(){
            if chip.is_loaded(){
                chip.run();
            }
            chip.screen.render(&mut main, &event);
            if monitor.is_some(){
                monitor.as_mut().unwrap().next();
            }
        }else{
            running = false;
        }
    }
}*/