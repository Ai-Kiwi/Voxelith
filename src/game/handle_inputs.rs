use std::sync::mpsc::Receiver;

use crate::{game::{Game, InputEvent, pixel::PixelTypes, world::{self, WorldData}}, utils::{Vec3, raycast_test}};

pub fn handle_user_inputs(world : &mut WorldData, player_position : &mut Vec3, input_event_rx : &mut Receiver<InputEvent>) -> bool {
    loop {
        let player_input_event = input_event_rx.try_recv();
        match player_input_event {
            Ok(player_input_update) => {
                match player_input_update {
                    InputEvent::Move(_vec2) => {
                        //world.entities.get_mut(&0).unwrap().physics.velocity.0;
                        //world.entities.get_mut(&0).unwrap().physics.velocity.2;
                    },
                    InputEvent::Jump => {
                        println!("jump");
                        //if world.entities.get(&0).unwrap().physics.grounded == true {
                        //    world.entities.get_mut(&0).unwrap().physics.velocity.1 = 6.0;
                        //}
                    },
                    InputEvent::Click(position, facing_dir) => {
                        let mut i = 0;
                        'rayloop : for ray in raycast_test(position, facing_dir.normalize()) {
                            i+=1;
                            if world.get_pixel_data(ray.x as i32,ray.y as i32,ray.z as i32) != PixelTypes::Air {
                                for x in -5..=5 {
                                    for y in -5..=5 {
                                        for z in -5..=5 {
                                            world.set_pixel_data((ray.x as i32 + x,ray.y as i32 + y ,ray.z as i32 + z), PixelTypes::Air);
                                        }
                                    }
                                }
                                break 'rayloop;
                            }
    
    
                            //world.set_pixel_data((ray.x as i32,ray.y as i32,ray.z as i32), world::PixelTypes::Stone);

                            if i > 500 {
                                break;
                            }
                        }
                    },
                    InputEvent::CameraPositionUpdate(vec3) => {
                        player_position.x = vec3.x;
                        player_position.y = vec3.y;
                        player_position.z = vec3.z;
                    },
                }
            },
            Err(err) => {
                match err {
                    std::sync::mpsc::TryRecvError::Empty => break,
                    std::sync::mpsc::TryRecvError::Disconnected => return false,
                }
            },
        }
    }
    true
}