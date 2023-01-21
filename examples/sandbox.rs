use std::{sync::Arc, any::TypeId};

use message_broker::Handle;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let handle = Handle::new();
    handle.publish(Arc::new("Test".to_string())).await?;
    handle.subscribe(TypeId::of::<String>()).await?;

    Ok(())

    // let mut data = anymap::AnyMap::new();
    // data.insert(Vec::<String>::new());
    // data.insert(Vec::<i32>::new());

    // let int = data.get_mut::<Vec<i32>>().unwrap();
    // for i in 0..10 {
    //     (*int).push(i);
    // }

    // let strn = data.get_mut::<Vec<String>>().unwrap();
    // strn.push("Hello".to_string());
    // strn.push("World".to_string());

    // let x = data.get::<Vec<i32>>();
    // let y = data.get::<Vec<String>>();
    // dbg!(x, y);
}
