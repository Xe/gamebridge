use crate::MTState;
use tokio::stream::StreamExt as _;
use twitchchat::{events, Control, Dispatcher, Runner, Status};

pub(crate) fn run(st: MTState) {
    use tokio::runtime::Runtime;
    Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(handle(st));
}

async fn handle(st: MTState) {
    let (nick, pass) = (
        // twitch name
        std::env::var("TWITCH_NICK").unwrap(),
        // oauth token for twitch name
        std::env::var("TWITCH_PASS").unwrap(),
    );

    // putting this in the env so people don't join my channel when running this
    let channels = &[std::env::var("TWITCH_CHANNEL").unwrap()];

    let dispatcher = Dispatcher::new();
    let (runner, control) = Runner::new(dispatcher.clone(), twitchchat::RateLimit::default());
    let fut = run_loop(control.clone(), dispatcher, channels, st);

    let conn = twitchchat::connect_easy_tls(&nick, &pass).await.unwrap();

    tokio::select! {
        _ = fut => { control.stop() }
        status = runner.run(conn) => {
            match status {
                Ok(Status::Eof) => {}
                Ok(Status::Canceled) => {}
                Ok(Status::Timeout) => {}
                Err(err) => panic!(err),
            }
        }
    }
}

async fn run_loop(
    mut control: Control,
    mut dispatcher: Dispatcher,
    channels: &[String],
    st: MTState,
) {
    let mut join = dispatcher.subscribe::<events::Join>();
    let mut part = dispatcher.subscribe::<events::Part>();
    let mut pmsg = dispatcher.subscribe::<events::Privmsg>();

    async fn wait_and_join(
        control: &mut Control,
        dispatcher: &mut Dispatcher,
        channels: &[String],
    ) {
        let ready = dispatcher.wait_for::<events::IrcReady>().await.unwrap();
        eprintln!("our name: {}", ready.nickname);

        let w = control.writer();
        for channel in channels {
            eprintln!("joining: {}", channel);
            let _ = w.join(channel).await;
            eprintln!("joined");
        }
        eprintln!("joined all channels")
    }

    wait_and_join(&mut control, &mut dispatcher, channels).await;

    loop {
        tokio::select! {
            Some(msg) = join.next() => {
                eprintln!("{} joined {}", msg.name, msg.channel);
            }
            Some(msg) = part.next() => {
                eprintln!("{} left {}", msg.name, msg.channel);
            }
            Some(msg) = pmsg.next() => {
                let chatline = msg.data.to_string();
                let chatline = chatline.to_ascii_lowercase();
                let mut data = st.write().unwrap();
                const BUTTON_ADD_AMT: i64 = 64;

                for cmd in chatline.to_string().split(" ").collect::<Vec<&str>>().iter() {
                    match *cmd {
                        "a" => data.a_button.add(BUTTON_ADD_AMT),
                        "b" => data.b_button.add(BUTTON_ADD_AMT),
                        "z" => data.z_button.add(BUTTON_ADD_AMT),
                        "r" => data.r_button.add(BUTTON_ADD_AMT),
                        "cup" => data.c_up.add(BUTTON_ADD_AMT),
                        "cdown" => data.c_down.add(BUTTON_ADD_AMT),
                        "cleft" => data.c_left.add(BUTTON_ADD_AMT),
                        "cright" => data.c_right.add(BUTTON_ADD_AMT),
                        "start" => data.start.add(BUTTON_ADD_AMT),
                        "up" => data.sticky.add(127),
                        "down" => data.sticky.add(-128),
                        "left" => data.stickx.add(-128),
                        "right" => data.stickx.add(127),
                        "stop" => {data.stickx.update(0); data.sticky.update(0);},
                        _ => {},
                    }
                }

                eprintln!("[{}] {}: {}", msg.channel, msg.name, msg.data);
            }

            else => { break }
        }
    }
}
