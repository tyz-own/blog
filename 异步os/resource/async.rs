use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};
use tokio::time::{self, Duration};
use std::time::Instant;

async fn long_running_task(id: u32) {
    println!("Task {} started", id);
    // 模拟一个需要较长时间的异步操作
    time::sleep(Duration::from_secs(2)).await;
    println!("Task {} completed", id);
}


async fn read_file_async(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    time::sleep(Duration::from_secs(1)).await;
    file.read_to_string(&mut contents).await?;
    println!("Async read completed");
    Ok(contents)
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let mut handles = vec![];
     // 创建多个异步任务
    for i in 0..5 {
        let handle = tokio::spawn(long_running_task(i));
        handles.push(handle);
    }
   
    // let handle = tokio::spawn(read_file_async("example.txt"));
    // handles.push(handle);
    // 异步读取文件内容
    match read_file_async("example.txt").await {
        Ok(contents) => println!("File contents:\n"),
        Err(e) => eprintln!("Error reading file: {}", e),
    }
     // 等待所有任务完成
     for handle in handles {
        let _ = handle.await;
    }
    let duration = start.elapsed();
    println!("Async read completed in: {:?}", duration);
}
