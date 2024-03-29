use crate::basics::{Id, id};

pub struct Coords {
    name: Id,
    rect: raylib::math::Rectangle,
    px: f32,
    py: f32,
    texture: Id,
}

use sxd_xpath::nodeset::Node;

trait Attr {
    fn attr<T: std::str::FromStr>(&self, attr: &str) -> T;
    fn attr_opt<T: std::str::FromStr>(&self, attr: &str, default: T) -> T;
}

impl Attr for sxd_document::dom::Element<'_> {
    fn attr_opt<T: std::str::FromStr>(&self, attr: &str, default: T) -> T {
        match self.attribute_value(attr).map(|x| x.parse::<T>()) {
            Some(Ok(v)) => v,
            _ => default,
        }
    }
    fn attr<T: std::str::FromStr>(&self, attr: &str) -> T {
        match self.attribute_value(attr).unwrap().parse::<T>() {
            Ok(v) => v,
            Err(_) => panic!("Unable to parse"),
        }
    }
}

impl Coords {
    fn from(node: &sxd_xpath::nodeset::Node, texture: Id) -> Option<Self> {
        match node {
            Node::Element(element) => {
                match element.name().local_part() {
                    "sprite" => {
                        let name: String = element.attribute_value("n").unwrap().into();
                        let x = element.attr::<usize>("x");
                        let y = element.attr::<usize>("y");
                        let width = element.attr::<usize>("w");
                        let height = element.attr::<usize>("h");
                        let px = element.attr_opt::<f32>("pX", 0.5);
                        let py = element.attr_opt::<f32>("pY", 0.5);
                        Some(Coords {
                            name: id(&name),
                            rect: raylib::math::Rectangle::new(
                                x as f32,
                                y as f32,
                                width as f32,
                                height as f32,
                            ),
                            px,
                            py,
                            texture,
                        })
                    }
                    "SubTexture" => {
                        let name: String = element.attribute_value("name").unwrap().into();
                        let x = element.attr::<usize>("x");
                        let y = element.attr::<usize>("y");
                        let width = element.attr::<usize>("width");
                        let height = element.attr::<usize>("height");
                        let px = element.attr_opt::<f32>("pX", 0.5);
                        let py = element.attr_opt::<f32>("pY", 0.5);
                        Some(Coords {
                            name: id(&name),
                            rect: raylib::math::Rectangle::new(
                                x as f32,
                                y as f32,
                                width as f32,
                                height as f32,
                            ),
                            px,
                            py,
                            texture,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

pub struct SpriteSheet {
    pub textures: std::collections::HashMap<Id, raylib::texture::Texture2D>,
    pub sprites: std::collections::HashMap<Id, Coords>,
}

// pub struct SpriteSheets(Vec<SpriteSheet>);

use sxd_document::{parser, Package};
use sxd_xpath::{evaluate_xpath, Value};

// use sxd_document::dom::*;

fn get_coords(root: Package, texture: Id) -> Vec<Coords> {
    let doc = root.as_document();
    let result = evaluate_xpath(&doc, "/*/SubTexture");
    let mut sprites: Vec<Coords> = match result {
        Ok(Value::Nodeset(nodes)) => nodes
            .document_order()
            .iter()
            .filter_map(|node| Coords::from(node, texture))
            .collect(),
        _ => panic!("Invalid xml file"),
    };
    let result = evaluate_xpath(&doc, "/*/sprite");
    match result {
        Ok(Value::Nodeset(nodes)) => sprites.append(
            &mut nodes
                .document_order()
                .iter()
                .filter_map(|node| Coords::from(node, texture))
                .collect::<Vec<Coords>>(),
        ),
        _ => panic!("Invalid xml file"),
    }
    sprites
    // let mut current = root.first_child().expect("No children");
    // loop {
    //   if current.name() == "SubTexture"
    //   match current.next_sibling() {
    //     None => break,
    //     Some(next) => current = next
    //   }
    // }
    // let mut out = vec![];
    // inner(root, &mut out);
    // out
}

use raylib::core::drawing::RaylibDraw;
use std::collections::HashMap;

impl SpriteSheet {
    pub fn new() -> Self {
        SpriteSheet {
            textures: HashMap::new(),
            sprites: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        rl: &mut raylib::RaylibHandle,
        rt: &raylib::RaylibThread,
        image_path: &str,
        xml_path: &str,
    ) {
        let image = rl.load_texture(rt, image_path).unwrap();
        let xml = parser::parse(&std::fs::read_to_string(xml_path).expect("xml file not found"))
            .expect("Invalid XML file");
        let id = id(image_path);
        let coords = get_coords(xml, id);
        self.textures.insert(id, image);
        for item in coords {
            self.sprites.insert(item.name, item);
        }
    }

    pub fn draw(
        &self,
        rd: &mut crate::draw::DrawHandle,
        sprite: &str,
        dest: (f32, f32),
        pivot_offset: (f32, f32),
        rotation: f32,
        height: f32,
        flip: bool,
    ) {
        let coords = self.sprites.get(sprite).expect("Sprite not found");
        let source = if flip {
            raylib::math::Rectangle {
                x: coords.rect.x, // + coords.rect.width,
                y: coords.rect.y,
                width: -coords.rect.width,
                height: coords.rect.height,
            }
        } else {
            coords.rect
        };
        let width = coords.rect.width as f32 / coords.rect.height as f32 * height;
        rd.draw_texture_pro(
            &self.textures.get(&coords.texture).unwrap(),
            source,
            raylib::math::Rectangle {
                x: dest.0,
                y: dest.1,
                width,
                height,
            },
            raylib::math::Vector2::from((
                width as f32 * (coords.px + pivot_offset.0),
                height as f32 * (coords.py + pivot_offset.1),
            )),
            rotation,
            raylib::color::Color::from((255, 255, 255, 255)),
        )
    }
}
