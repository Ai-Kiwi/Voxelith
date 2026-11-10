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
    pub const fn is_transparent(self) -> bool {
        match self {
            PixelTypes::Water => true,
            PixelTypes::Air
            | PixelTypes::Grass
            | PixelTypes::Dirt
            | PixelTypes::Stone
            | PixelTypes::Sand
            | PixelTypes::Snow => false,
        }
    }
    pub const fn should_be_face(self, self_transparent: bool) -> bool {
        if self_transparent {
            match self {
                PixelTypes::Air => true,
                PixelTypes::Grass
                | PixelTypes::Dirt
                | PixelTypes::Stone
                | PixelTypes::Water
                | PixelTypes::Sand
                | PixelTypes::Snow => false,
            }
        } else {
            match self {
                PixelTypes::Air | PixelTypes::Water => true,
                PixelTypes::Grass
                | PixelTypes::Dirt
                | PixelTypes::Stone
                | PixelTypes::Sand
                | PixelTypes::Snow => false,
            }
        }
    }
    pub const fn is_solid(self) -> bool {
        match self {
            PixelTypes::Air | PixelTypes::Water => false,
            PixelTypes::Grass
            | PixelTypes::Dirt
            | PixelTypes::Stone
            | PixelTypes::Sand
            | PixelTypes::Snow => true,
        }
    }
}

#[test]
fn test_transparent_pixel() {
    assert!(PixelTypes::Water.is_transparent());
    assert!(!PixelTypes::Air.is_transparent());
    assert!(!PixelTypes::Grass.is_transparent());
    assert!(!PixelTypes::Dirt.is_transparent());
    assert!(!PixelTypes::Stone.is_transparent());
    assert!(!PixelTypes::Sand.is_transparent());
    assert!(!PixelTypes::Snow.is_transparent());
}

#[test]
fn test_solid_pixel() {
    assert!(!PixelTypes::Water.is_solid());
    assert!(!PixelTypes::Air.is_solid());
    assert!(PixelTypes::Grass.is_solid());
    assert!(PixelTypes::Dirt.is_solid());
    assert!(PixelTypes::Stone.is_solid());
    assert!(PixelTypes::Sand.is_solid());
    assert!(PixelTypes::Snow.is_solid());
}

#[test]
fn test_visible_face_pixel() {
    assert!(PixelTypes::Air.should_be_face(true));
    assert!(PixelTypes::Air.should_be_face(false));

    assert!(!PixelTypes::Water.should_be_face(true));
    assert!(PixelTypes::Water.should_be_face(false));

    assert!(!PixelTypes::Stone.should_be_face(true));
    assert!(!PixelTypes::Stone.should_be_face(false));
}
