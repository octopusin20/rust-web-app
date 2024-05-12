/*! `/customers` */

use rocket::{
    response::status::Created,
    serde::json::Json,
    State,
};

use crate::{
    api::error::{
        self,
        Error,
    },
    domain::{
        customers::*,
        infra::*,
    },
};

/** `GET /customers/<id>` */
#[get("/<id>")]
pub async fn get(id: CustomerId, app: &State<App>) -> Result<Json<CustomerWithOrders>, Error> {
    app.transaction(|app| async move {
        let query = app.get_customer_with_orders_query();

        match query.execute(GetCustomerWithOrders { id }).await? {
            Some(customer) => Ok(Json(customer)),
            None => Err(Error::NotFound(error::msg("customer not found"))),
        }
    })
    .await
}

/** `PUT /customers` */
#[put("/", format = "application/json")]
pub async fn create(app: &State<App>) -> Result<Created<Json<CustomerId>>, Error> {
    app.transaction(|app| async move {
        let id = app.customer_id();

        let command = app.create_customer_command();

        let id = id.get()?;

        command.execute(CreateCustomer { id }).await?;

        let location = format!("/customers/{}", id);

        Ok(Created::new(location).body(Json(id)))
    })
    .await
}

fn serial_rx_to_nats(port: &mut BoxSerial, nats_client: &mut nats::Client) -> BoxResult<()> {
    let mut buffer: Vec<u8> = vec![0; 1024];

    // Send data from serial to NATS forever
    loop {
        if let Ok(length) = port.read(&mut buffer.as_mut_slice()) {
            if length > 0 {
                nats_client.publish("radar", &buffer)?;
            }
        }
    }
}
