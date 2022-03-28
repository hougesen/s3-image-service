FROM rust:1.59 as builder 
WORKDIR /usr/src/s3-image-service
COPY . . 
RUN cargo install --path . 

FROM debian:buster-slim as runner
RUN apt-get update && apt-get install -y build-essential && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/s3-image-service /usr/local/bin/s3-image-service
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["s3-image-service"]
