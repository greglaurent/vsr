#[allow(dead_code, unused)]
use axum::{response::Html, routing::get, Router};
use viewstamp::Replica;

pub mod viewstamp {
    use std::{
        collections::{HashMap, VecDeque},
        net::SocketAddr,
        sync::atomic::{AtomicU64, AtomicUsize},
    };

    use scru128::Scru128Id;

    pub type Id = Scru128Id;
    pub type Configuration = VecDeque<SocketAddr>;
    pub type RequestId = AtomicU64;
    pub type Log = VecDeque<Request>;

    pub fn new_id() -> Id {
        scru128::new()
    }

    pub struct Request(u64);
    pub struct Response(String);

    pub enum RequestStatus {
        Sent,
        Pending,
        Closed,
    }

    pub struct Record {
        pub op: RequestId,
        pub request: Request,
        pub status: RequestStatus,
        pub result: Response,
    }

    pub enum ReplicaStatus {
        Normal,
        ViewChange,
        Recovering,
    }

    pub struct Replica {
        pub id: Id,
        pub config: Configuration,
        pub repl_num: AtomicUsize,
        pub curr_view: AtomicUsize,
        pub curr_op: RequestId,
        pub last_op: RequestId,
        pub client_table: HashMap<Id, Vec<Record>>,
        status: ReplicaStatus,
    }

    impl Replica {
        pub fn new(ip: SocketAddr) -> Self {
            let mut repl = Self {
                id: new_id(),
                config: VecDeque::new(),
                repl_num: AtomicUsize::default(),
                curr_view: AtomicUsize::default(),
                curr_op: AtomicU64::default(),
                last_op: AtomicU64::default(),
                client_table: HashMap::default(),
                status: ReplicaStatus::ViewChange,
            };

            repl.config.push_back(ip);

            assert!(repl.config.len() == 1);
            assert!(repl.config.get(0) == Some(&ip));

            assert!(repl.repl_num.load(std::sync::atomic::Ordering::SeqCst) == 0);
            assert!(repl.curr_op.load(std::sync::atomic::Ordering::SeqCst) == 0);
            assert!(repl.last_op.load(std::sync::atomic::Ordering::SeqCst) == 0);
            assert!(repl.client_table.len() == 0);

            repl
        }

        fn view_change(&mut self) {}
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let ip = listener.local_addr().unwrap();

    let _ = Replica::new(ip);

    axum::serve(listener, app).await.unwrap();
}

pub async fn handler() -> Html<&'static str> {
    Html("<h1>Hello World</h1>")
}
