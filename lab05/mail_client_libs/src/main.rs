use clap::{Parser, ValueEnum};
use lettre::message::{header::ContentType, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Txt,
    Html,
}

#[derive(Parser, Debug)]
#[command(about = "mail client libs")]
struct Args {
    #[arg(short, long)]
    to: String,

    #[arg(short, long, default_value = "Default subject")]
    subject: String,

    #[arg(short, long, default_value = "Hello world!")]
    body: String,

    #[arg(short, long, value_enum, default_value = "txt")]
    format: Format,

    #[arg(long, default_value = "smtp.gmail.com")]
    smtp_host: String,

    #[arg(long, default_value_t = 587)]
    smtp_port: u16,

    #[arg(long, default_value = "sender@example.com")]
    from: String,

    #[arg(long)]
    username: Option<String>,

    #[arg(long)]
    password: Option<String>,
}

fn main() {
    let args = Args::parse();

    let content_type = match args.format {
        Format::Txt => ContentType::TEXT_PLAIN,
        Format::Html => ContentType::TEXT_HTML,
    };

    let body = match args.format {
        Format::Txt => args.body.clone(),
        Format::Html => {
            if args.body == "Hello world!" {
                "<html><body><h1>Hello world!</h1><p>This is an <b>HTML</b> email sent via the mail client.</p></body></html>".to_string()
            } else {
                args.body.clone()
            }
        }
    };

    let email = Message::builder()
        .from(args.from.parse().expect("Invalid sender address"))
        .to(args.to.parse().expect("Invalid recipient address"))
        .subject(&args.subject)
        .singlepart(SinglePart::builder().header(content_type).body(body))
        .expect("Failed to build email");

    let transport = if let (Some(user), Some(pass)) = (args.username, args.password) {
        let creds = Credentials::new(user, pass);
        SmtpTransport::starttls_relay(&args.smtp_host)
            .expect("Failed to create SMTP transport")
            .port(args.smtp_port)
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::relay(&args.smtp_host)
            .expect("Failed to create SMTP transport")
            .port(args.smtp_port)
            .build()
    };

    match transport.send(&email) {
        Ok(_) => println!("Email sent successfully to {}!", args.to),
        Err(e) => eprintln!("Failed to send email: {e}"),
    }
}
