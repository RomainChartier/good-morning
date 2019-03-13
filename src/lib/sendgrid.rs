use reqwest::*;

use serde_derive;
use super::common::GoodMorningError;

#[derive(Serialize, Debug)]
struct Personalization{
    to: Vec<MailAddress>,
    subject: String
}

#[derive(Serialize, Debug)]
struct MailAddress{
    email: String,
}

#[derive(Serialize, Debug)]
struct MailContent{
    r#type: String,
    value: String,
}

#[derive(Serialize, Debug)]
struct MailRequest{
    personalizations: Vec<Personalization>,
    from: MailAddress,
    content: Vec<MailContent>
}


pub fn send_test_mail() {
   
    let req = MailRequest{
        personalizations: vec!(
            Personalization {
                subject: "Test!!".to_string(),
                to: vec!(MailAddress {
                    email: "".to_string()
                })
            }
        ),
        from: MailAddress { email: "test@chartier.com".to_string() },
        content: vec!(MailContent {
            r#type: "text/plain".to_string(),
            value: "Salut rom, ça va?".to_string()
        })
    };

    let request_url = format!("https://api.sendgrid.com/v3/mail/send");
    println!("{}", request_url);


    let mut response = dbg!(Client::new()
        .post(&request_url)
        .bearer_auth("".to_string())
        .json(&req)
        .send()
        ).expect("");

    println!("{:?}", response.text());
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

