use reqwest::Client;

use crate::common::GoodMorningError;

#[derive(Serialize, Debug)]
pub struct Personalization {
    to: Vec<MailAddress>,
    subject: String,
}

#[derive(Serialize, Debug)]
pub struct MailAddress {
    email: String,
}

#[derive(Serialize, Debug)]
pub struct MailContent {
    r#type: String,
    value: String,
}

#[derive(Serialize, Debug)]
pub struct MailRequest {
    personalizations: Vec<Personalization>,
    from: MailAddress,
    content: Vec<MailContent>,
}

impl MailRequest {
    pub fn new(subject: &str, to_email: &str, from_email: &str, content: &str) -> MailRequest {
        MailRequest {
            personalizations: vec![Personalization {
                subject: subject.to_string(),
                to: vec![MailAddress {
                    email: to_email.to_string(),
                }],
            }],
            from: MailAddress {
                email: from_email.to_string(),
            },
            content: vec![MailContent {
                r#type: "text/plain".to_string(),
                value: content.to_string(),
            }],
        }
    }
}

pub fn send_mail(api_token: &str, mail_request: &MailRequest) -> Result<(), GoodMorningError> {
    let request_url = "https://api.sendgrid.com/v3/mail/send".to_string();

    let mut response = Client::new()
        .post(&request_url)
        .bearer_auth(api_token)
        .json(mail_request)
        .send()?;

    println!("{:?}", response.text());

    Ok(())
}

// Request

// POST https://api.sendgrid.com/v3/mail/send HTTP/1.1

// Request Body

// {
//   "personalizations": [
//     {
//       "to": [
//         {
//           "email": "john@example.com"
//         }
//       ],
//       "subject": "Hello, World!"
//     }
//   ],
//   "from": {
//     "email": "from_address@example.com"
//   },
//   "content": [
//     {
//       "type": "text/plain",
//       "value": "Hello, World!"
//     }
//   ]
// }

// Response

// {
//   HTTP/1.1 202
// }
