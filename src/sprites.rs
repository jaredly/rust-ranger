

pub struct Coords {
  name: String,
  x: usize,
  y: usize,
  width: usize,
  height: usize,
  px: f32,
  py: f32,
  texture: String,
}

use sxd_xpath::nodeset::Node;

impl Coords {
  fn from(node: &sxd_xpath::nodeset::Node, texture: &str) -> Option<Self> {
    match node {
      Node::Element(element) => {
        if element.name().local_part() != "SubTexture" {
          return None
        }
        let name: String = element.attribute_value("name").unwrap().into();
        let x = element.attribute_value("x").unwrap().parse::<usize>().unwrap();
        let y = element.attribute_value("y").unwrap().parse::<usize>().unwrap();
        let width = element.attribute_value("width").unwrap().parse::<usize>().unwrap();
        let height = element.attribute_value("height").unwrap().parse::<usize>().unwrap();
        let px = element.attribute_value("pX").map(|v| v.parse::<f32>().unwrap()).unwrap_or(0.5);
        let py = element.attribute_value("pY").map(|v| v.parse::<f32>().unwrap()).unwrap_or(0.5);
        Some(Coords {name, x, y, width, height, px, py, texture: texture.to_owned()})
      }
      _ => None
    }
  }
}

pub struct SpriteSheet {
  pub textures: std::collections::HashMap<String, raylib::texture::Texture2D>,
  pub sprites: std::collections::HashMap<String, Coords>
}

// pub struct SpriteSheets(Vec<SpriteSheet>);

use sxd_document::{parser, Package};
use sxd_xpath::{evaluate_xpath, Value};

// use sxd_document::dom::*;

fn get_coords(root: Package, texture: &str) -> Vec<Coords> {
  let doc = root.as_document();
  let result = evaluate_xpath(&doc, "/*/SubTexture");
  match result {
    Ok(Value::Nodeset(nodes)) => {
      nodes.document_order().iter().filter_map(|node| Coords::from(node, texture)).collect()
    }
    _ => panic!("Invalid xml file")
  }
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

use std::collections::HashMap;
use raylib::core::drawing::RaylibDraw;

impl SpriteSheet {
  pub fn new() -> Self {
    SpriteSheet {
      textures: HashMap::new(),
      sprites: HashMap::new(),
    }
  }

  pub fn add(&mut self, rl: &mut raylib::RaylibHandle, rt: &raylib::RaylibThread, image_path: &str, xml_path: &str) {
    let image = rl.load_texture(rt, image_path).unwrap();
    let xml = parser::parse(&std::fs::read_to_string(xml_path).expect("xml file not found")).expect("Invalid XML file");
    let coords = get_coords(xml, image_path);
    self.textures.insert(image_path.to_owned(), image);
    for item in coords {
      self.sprites.insert(item.name.clone(), item);
    }
    // SpriteSheet {
    //   textures: image,
    //   sprites: std::collections::HashMap::from_iter(coords.into_iter().map(|item| (item.name.clone(), item)))
    // }
  }
  
  pub fn draw(&self, rd: &mut raylib::drawing::RaylibDrawHandle<raylib::RaylibHandle>, sprite: &String, dest: (f32, f32), rotation: f32, height: f32) {
    let coords = self.sprites.get(sprite).expect("Sprite not found");
    let source = raylib::math::Rectangle {x: coords.x as f32, y: coords.y as f32, width: coords.width as f32, height: coords.height as f32};
    let width = coords.width as f32 / coords.height as f32 * height;
    rd.draw_texture_pro(&self.textures.get(&coords.texture).unwrap(), source, raylib::math::Rectangle {
      x: dest.0,
      y: dest.1,
      // width: coords.width as f32,
      // height: coords.height as f32,
      // width: coords.width as f32 / coords.height as f32 * height,
      width,
      height,
    }, raylib::math::Vector2::from((width as f32 * coords.px, height as f32 * coords.py)), rotation, raylib::color::Color::from((255, 255, 255, 255)))
  }
}
