error_chain! {
    errors {
        /// An error meaning you sent a bad request to reddit
        BadRequest {
            description("Reddit failed to handle the request")
            display("Failed to handle request")
        }

        /// You tried to perform an action that was unauthorized. Make sure to set the auth field
        /// of a connection
        Unauthorized {
            description("A request was made without proper authorization")
            display("Made an unauthorized request")
        }

        /// The response recieved was formatted in an unexpected way and failed to parse
        ResponseError(response: String) {
            description("Got a response that was unexpected")
            display("Unexpected response: {}", response)
        }

        Unimplemented {
            description("Tried to do something unimplemented")
            display("Unimplented feature")
        }

        InvalidJson(jsonstr: String) {
            description("JSON recieved could not be parsed correctly")
            display("Invalid json: {}", jsonstr)
        }
    }
}
