use serde_json::Value;
use handlebars::{Handlebars, HelperDef, RenderContext, Helper, Context, JsonRender, HelperResult, Output, RenderError};
use sqlx::Row;

#[derive(Clone, Copy)]
pub struct SimpleHelper;

impl HelperDef for SimpleHelper {
  fn call<'reg: 'rc, 'rc>(&self, h: &Helper, _: &Handlebars, _: &Context, rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    // let param = h.param(0).unwrap();

    // out.write("1st helper: ")?;
    // out.write(param.value().render().as_ref())?;
    let p = h.param(0).unwrap();
    
    // info!("Value: {:?}", p.value());

    // if let Value::Object(map) = p.value() {
    //     if map.contains_key("File") {
    //         let file = map.get("File").unwrap();
    //         let name = file.get("name").unwrap().as_str().unwrap();
    //         out.write(&format!("File: '{}", name))?;
    //     } else if map.contains_key("Text") {
    //         let file = map.get("Text").unwrap();
    //         let content = file.get("content").unwrap();
    //         let r = content.render();
    //         out.write(&format!("<i>{}</i>", r.as_str()))?;
    //     } else if map.contains_key("Image") {
    //         out.write("Image")?;
    //     }
    // }
    Ok(())
  }
}