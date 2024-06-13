# Untested


USER cargo

# Copy the source code into the container
COPY --chown=cargo:cargo . .

# Build the application
RUN cargo build --release

# Run the application
CMD ["./target/release/finance-fusion-server"]
