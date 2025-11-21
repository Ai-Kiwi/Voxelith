
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PixelTypes {
    Air,
    Grass,
    Dirt,
    Stone,
    Water,
    Sand,
    Snow,
}

impl PixelTypes {
    pub const fn is_transparent(&self) -> bool {
        match self {
            PixelTypes::Air => false,
            PixelTypes::Grass => false,
            PixelTypes::Dirt => false,
            PixelTypes::Stone => false,
            PixelTypes::Water => true,
            PixelTypes::Sand => false,
            PixelTypes::Snow => false,
        }
    }   
    pub const fn should_be_face(&self, self_transparent : bool) -> bool {
        if self_transparent {
            match self {
                PixelTypes::Air => true,
                PixelTypes::Grass => false,
                PixelTypes::Dirt => false,
                PixelTypes::Stone => false,
                PixelTypes::Water => false,
                PixelTypes::Sand => false,
                PixelTypes::Snow => false,
            }
        }else{
            match self {
                PixelTypes::Air => true,
                PixelTypes::Grass => false,
                PixelTypes::Dirt => false,
                PixelTypes::Stone => false,
                PixelTypes::Water => true,
                PixelTypes::Sand => false,
                PixelTypes::Snow => false,
            }            
        }

    }
}