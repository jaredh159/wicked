use crate::internal::*;

const PARALLELISM: u32 = 10;

pub async fn run(shared_db: Arc<Mutex<DbClient>>) -> Result<()> {
  static NUM_COMPLETED: AtomicU32 = AtomicU32::new(0);
  let total = 167_300_740; // todo: pass
  let sample_size: u32 = 10_000; // todo: pass
  assert!(sample_size > PARALLELISM);

  let db = shared_db.clone();
  let db = db.lock().await;
  let row = db.query_one(COUNT_COMPLETE, &[]).await?;
  drop(db);

  let db_count: u32 = row.get::<_, String>(0).parse().unwrap();
  NUM_COMPLETED.store(db_count, Ordering::Relaxed);

  let tasks = stream::until_completed(sample_size, &NUM_COMPLETED)
    .map(|()| {
      tokio::spawn({
        let client = shared_db.clone();
        async move {
          if NUM_COMPLETED.load(Ordering::Relaxed) >= (sample_size - PARALLELISM + 1) {
            return Ok::<(), Error>(());
          }

          let domain = db::random_unchecked_domain(client.clone(), total).await?;
          println!("checking: {domain}");
          NUM_COMPLETED.fetch_add(1, Ordering::Relaxed);
          Ok(())
        }
      })
    })
    .buffer_unordered(PARALLELISM as usize);

  tasks.collect::<Vec<_>>().await;
  let final_count = NUM_COMPLETED.load(Ordering::Acquire);
  println!("final count: {final_count}");
  Ok(())
}

static SELECT_RANDOM: &str = "SELECT domain FROM domains WHERE id =";
static SELECT_CHECKED: &str = "SELECT id FROM checked WHERE domain =";
static COUNT_COMPLETE: &str =
  "SELECT COUNT(*)::text FROM checked WHERE status != 'unreachable'";
