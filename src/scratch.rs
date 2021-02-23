#[derive(Default)]
struct TimeWatcher(Option<String>);

#[async_trait]
impl Watch for TimeWatcher {
    async fn watch(&self, tx_updates: UnboundedSender<Update>) {
        tracing::trace!("TimeWatcher::watch");
        let time_format = "%b %d %A %l:%M:%S %p";
        let mut interval = time::interval(Duration::from_millis(1_000));

        loop {
            interval.tick().await;

            let now: DateTime<Local> = Local::now();
            let fmt_now = now.format(time_format).to_string();

            tx_updates.send(Update::Time(Some(fmt_now))).unwrap();
            tx_updates.send(Update::Redraw).unwrap();
        }
    }
}

    /*
    let mut wg = WatchGroup::new();
    wg.push(&TimeWatcher(None));
    let mut rx_updates = wg.start();
    */

#[async_trait]
trait Watch {
    async fn watch(&self, tx_updates: UnboundedSender<Update>);
}

struct WatchGroup {
    rx_updates: UnboundedReceiver<Update>,
    tx_updates: UnboundedSender<Update>,
    join_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl WatchGroup {
    fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self {
            rx_updates: rx,
            tx_updates: tx,
            join_handles: Vec::new(),
        }
    }

    fn push(&mut self, watcher: &'static (impl Watch + Sync)) {
        let handle = tokio::spawn(watcher.watch(self.tx_updates.clone()));
        self.join_handles.push(handle);
    }

    fn start(self) -> UnboundedReceiver<Update> {
        let futures_stream = futures::stream::iter(self.join_handles);
        let mut worker_errors = futures_stream.buffer_unordered(5);
        tokio::spawn(async move {
            while let Some(error) = worker_errors.next().await {
                tracing::error!("{:?}", error);
            }
        });

        self.rx_updates
    }
}



struct ChannelWriter(UnboundedSender<Vec<u8>>);

impl std::io::Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        self.0.send(Vec::from(buf)).unwrap();
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}
