FROM rust:1.59 as builder 
WORKDIR /app
COPY . . 
RUN cargo install --path . 

FROM debian:buster-slim as runner
COPY --from=builder /usr/local/cargo/bin/s3-image-service /usr/local/bin/s3-image-service
ENV ROCKET_ADDRESS=0.0.0.0
ENV AWS_ACCESS_KEY_ID=
ENV AWS_SECRET_ACCESS_KEY=
ENV BUCKET_NAME=
EXPOSE 8000
CMD ["s3-image-service"]