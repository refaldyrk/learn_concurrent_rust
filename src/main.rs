fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::sync::{Arc, Barrier, Mutex};
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_create_thread() {
        thread::spawn(|| {
            for i in 1..10 {
                println!("counter {}", i);
                thread::sleep(Duration::from_secs(1));
            }
        });

        println!("Application finished");
        thread::sleep(Duration::from_secs(7));
    }

    #[test]
    fn test_join_thread() {
        let handler = thread::spawn(|| {
            let mut counter = 0;
            for i in 1..10 {
                println!("counter {}", counter);
                thread::sleep(Duration::from_secs(1));
                counter += 1;
            }

            return counter
        });

        let result = handler.join();
        match result {
            Ok(result) => println!("Result: {:?}", result),
            Err(err) => println!("Error: {:?}", err)
        }

        println!("Application finished");
    }

    fn looping_five_counter() -> i32 {
        let mut counter = 0;

        for i in 1..6 {
            println!("counter {}", counter);
            thread::sleep(Duration::from_secs(1));
            counter += 1;
        };

        return counter
    }

    #[test]
    fn test_looping_five_counter_parallel() {
        let seq1 = thread::spawn(|| looping_five_counter());
        let seq2 = thread::spawn(|| looping_five_counter());

        let result1 = seq1.join();
        let result2 = seq2.join();

        match result1 {
            Ok(result) => println!("Result: {:?}", result),
            Err(err) => println!("Error: {:?}", err)
        }

        match result2 {
            Ok(result) => println!("Result: {:?}", result),
            Err(err) => println!("Error: {:?}", err)
        }

        println!("Application finished");
    }

    #[test]
    fn test_factory_thread() {
        let factory = thread::Builder::new().name("Factory".to_string());

        let handle = factory.spawn(|| looping_five_counter())
            .expect("Failed to create thread");

        handle.join()
            .unwrap();

        println!("Application finished");
    }

    #[test]
    fn channel_thread() {
        let (sender, receiver) = std::sync::mpsc::channel::<String>();

        thread::spawn(move || {
            sender.send("Hello".to_string()).unwrap();
        });

        thread::spawn(move || {
            let result = receiver.recv();
            match result {
                Ok(result) => println!("Result: {:?}", result),
                Err(err) => println!("Error: {:?}", err)
            }
        });

        println!("Application finished");

        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    fn atomic_thread_safely_race_condition() {
        let counter = Arc::new(AtomicI32::new(0));

        let mut handlers = vec![];

        for _ in 0..10 {
            let clone = Arc::clone(&counter);
            let handler = thread::spawn(move || {
                for j in 0..10 {
                    clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            handlers.push(handler);
        }

        for handler in handlers {
            handler.join().unwrap();
        }

        println!("Result: {}", counter.load(Ordering::Relaxed));
    }

    #[test]
    fn mutex_thread() {
        let counter = Arc::new(Mutex::new(0));

        let mut handlers = vec![];

        for _ in 0..10 {
            let clone = Arc::clone(&counter);
            let handler = thread::spawn(move || {
                for j in 0..10 {
                   let mut data = clone.lock().unwrap();
                   *data += 1;
                }
            });
            handlers.push(handler);
        }

        for handler in handlers {
            handler.join().unwrap();
        }

        println!("Result: {}", *counter.lock().unwrap());
    }

    thread_local! {
        pub static NAMES: RefCell<String> = RefCell::new("John Doe".to_string());
    }

    #[test]
    fn thread_local() {
        let handler = thread::spawn(|| {
            NAMES.with_borrow_mut(|name| {
                *name = "Jane Doe".to_string();
            });

            NAMES.with_borrow(|name| {
                println!("Name: {}", name);
            });
        });

        handler.join().unwrap();

        NAMES.with_borrow(|name| {
            println!("Name: {}", name);
        });
    }

    #[test]
    fn test_barrier() {
        let barrier = Arc::new(Barrier::new(10));
        let mut handlers = vec![];

        for i in 0..10 {
            let barrier_clone = Arc::clone(&barrier);

            let handler = thread::spawn(move || {
                println!("Thread {} waiting", i);
                barrier_clone.wait();
                println!("Thread {} finished", i);
            });

            handlers.push(handler);
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }

    use std::sync::Once;
    use tokio::runtime::Runtime;

    static mut TOTAL_COUNTER: i32 = 0;
    static ONCE_COUNTER: Once = Once::new();

    fn get_total() -> i32 {
        unsafe {
            ONCE_COUNTER.call_once(|| {
                TOTAL_COUNTER += 1;
            });

            return TOTAL_COUNTER
        }
    }
    
    #[test]
    fn test_once() {
        let mut handlers = vec![];

        for _ in 0..10 {
            let handler = thread::spawn(|| {
                let total = get_total();
                println!("Total: {}", total);
            });

            handlers.push(handler);
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }

    async fn get_async_data() -> String {
        thread::sleep(Duration::from_secs(2));
        return "Async data".to_string();
    }

    #[tokio::test]
    async fn test_async() {
        let data = get_async_data().await;
        println!("Data: {}", data);
    }

    async fn get_database_data(time: u64) -> String {
        println!("get database data {:?}", thread::current().id());
        tokio::time::sleep(Duration::from_secs(time)).await;
        println!("hello database {:?}", thread::current().id());
        return "Database data".to_string();
    }

    #[tokio::test]
    async fn test_database() {
       let mut handlers = vec![];
        for i in 0..5 {
            let handler = tokio::spawn(get_database_data(i));

            handlers.push(handler);
        }

        for handler in handlers {
            let data = handler.await.unwrap();
            println!("Data: {:?}", data);
        }
    }

    async fn run_concurrent(runtime: Arc<Runtime>) {
        let mut handlers = vec![];

        for i in 0..5 {
            let handler = runtime.spawn(get_database_data(i));

            handlers.push(handler);
        }

        for handler in handlers {
            let data = handler.await.unwrap();
            println!("Data: {:?}", data);
        }
    }
    
    #[test]
    fn runtime_concurrent() {
        let runtime = Arc::new(
          tokio::runtime::Builder::new_multi_thread()
              .worker_threads(10)
              .enable_time()
              .build()
              .unwrap()
        );

        runtime.block_on(run_concurrent(Arc::clone(&runtime)));
    }
}