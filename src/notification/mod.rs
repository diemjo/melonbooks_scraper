use webhook::client::{WebhookClient};

use crate::{model::Product, CONFIGURATION};
use crate::common::error::{Result, Error};

const IMAGE_URL: &str = "https://karpador.moe/images/favicon.png";
const DISCORD_URL: &str = "https://discord.com/api/webhooks/";

pub async fn notify_new_products<T: AsRef<Product>>(products: &[T], artist: &str) -> Result<()> {
    if CONFIGURATION.discord_api_key.is_none() {
        return Ok(());
    }
    if products.len()==0 {
        return Ok(());
    }
    let url = format!("{}{}", DISCORD_URL, CONFIGURATION.discord_api_key.as_ref().unwrap());
    let client: WebhookClient = WebhookClient::new(&url);
    for product_chunk in products.chunks(5) {
        client.send(|mut message| {
            message = message
                .content(&format!("New products for {}:", artist))
                .username("MelonbookScraper")
                .avatar_url(IMAGE_URL);
            for product in product_chunk {
                let product = product.as_ref();
                message = message
                    .embed(|embed| embed
                        .title(&product.title)
                        .description(&product.url)
                        .thumbnail(&product.img_url)
                    );
            }
            message
        }).await.or_else(|e| Err(Error::DiscordError(e.to_string())))?;
        tokio::time::sleep(core::time::Duration::from_secs(1)).await;
    }
    Ok(())
}

pub async fn notify_product_rerun(product: &Product) -> Result<()> {
    if CONFIGURATION.discord_api_key.is_none() {
        return Ok(());
    }
    let url = format!("{}{}", DISCORD_URL, CONFIGURATION.discord_api_key.as_ref().unwrap());
    let client: WebhookClient = WebhookClient::new(&url);
    client.send(|message| message
        .content("Products available again:")
        .username("MelonbookScraper")
        .avatar_url(IMAGE_URL)
        .embed(|embed| embed
            .title(&product.title)
            .description(&product.url)
            .thumbnail(&product.img_url))
    ).await.or_else(|e| Err(Error::DiscordError(e.to_string())))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use chrono::{Utc};

    use crate::model::Product;

    use super::notify_new_products;


    #[tokio::test]
    async fn test_send() {
        notify_new_products(&vec![
            Product::new(
                "https://www.melonbooks.co.jp/detail/detail.php?product_id=1793662".to_string(),
                "アクリルキューブ nana Uribou New Yaer2023".to_string(),
                "nana".to_string(),
                "https://melonbooks.akamaized.net/user_data/packages/resize_image.php?image=217001225510.jpg".to_string(),
                Utc::now().date_naive(),
                crate::model::Availability::Available
            ),
            Product::new(
                "https://www.melonbooks.co.jp/detail/detail.php?product_id=1704677".to_string(),
                "【2次受注】A3キャラファイングラフ nana 冬の物語(一般差分)".to_string(),
                "nana".to_string(),
                "https://melonbooks.akamaized.net/user_data/packages/resize_image.php?image=217001211823.jpg".to_string(),
                Utc::now().date_naive(),
                crate::model::Availability::Available
            )
        ] as &Vec<Product>, "nana").await.unwrap();
    }
}