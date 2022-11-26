use crate::chip::screen::renderer::Batch;
use crate::chip::screen::renderer::Disposable;

pub mod renderer;

pub struct Screen{ pixels : [[bool; 128]; 64], width: u32, height: u32, extended: bool, batch: Box<Batch>, data: Vec<f32> }

impl Screen{
    pub fn new(width: u32, height: u32)->Self{
        let batch = unsafe{
            Box::new(Batch::new(128 * 64))
        };
        
        return Screen{ pixels: [[false; 128]; 64], width, height, extended: true, batch, data: Vec::new()  }
    }

    pub fn clear(&mut self){
        for row in 0..self.pixels.len(){
            for column in 0..self.pixels[row].len(){
                self.pixels[row][column] = false;
            }
        }
    }

    pub fn is_extended(&self)->bool{ return self.extended; }

    pub fn set(&mut self, x:usize, y:usize){
        self.pixels[y][x] = true;
    }

    pub fn resize(&mut self, width: u32, height: u32){
        self.height = height;
        self.width = width;
    }

    pub fn render(&mut self){
        self.data.clear();
        for row in 0..self.pixels.len(){
            for column in 0..self.pixels[row].len(){
                if self.pixels[row][column]{
                    self.data.push(column as f32);
                    self.data.push(row as f32);
                }
            }
        }
        unsafe{
            self.batch.draw(self.pixels.len() as f32, self.pixels[0].len() as f32, &self.data);
        }
    }

    pub fn draw(&mut self, x: u16, y: u16, sprite:Vec<u16>)->u8{
        let mut vf = 0;
        for yline in 0..sprite.len(){
            for xline in 0..8{
                if (sprite[yline] & (0x80 >> xline)) != 0{
                    let (mut screen_x, mut screen_y) = (x as usize + xline, y as usize + yline);
                    while screen_x >= 64{ screen_x -= 64; }
                    while screen_y >= 32{ screen_y -= 32; }

                    if self.pixels[screen_y][screen_x] && vf == 0{
                        vf = 1;                   
                    }
                    self.pixels[screen_y][screen_x] ^= true;
                }
            }
        }
        return vf;
    }

    /*pub fn schip_draw(&mut self, x: u16, y: u16, sprite:Vec<u16>)->u8{
        let mut vf: u8 = 0;
        let (mut xi, mut yi) : (u16, u16);

        for yline in 0..16{
            yi = y + yline;
            for i in 0..16{
                xi = x + i;
                if (sprite[yline as usize] & (0x8000 >> i)) != 0{
                    if xi < 128 && yi < 64{
                        if self.pixels[yi as usize][xi as usize]{
                            vf = 1;                   
                        }
                        self.pixels[yi as usize][xi as usize] ^= true;
                    }
                }
            }
        }
        return vf;
    }*/
}

impl Disposable for Screen{
    unsafe fn dispose(&mut self) {
        self.batch.dispose();
    }
}