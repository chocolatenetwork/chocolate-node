# If this works, use this in gh actions to generate image instead
# FROM paritytech/ci-linux:595b5691-20211221
FROM islami00/choc-test:t1

WORKDIR /node

EXPOSE 9944

# CMD ["./target/release/chocolate","--dev","--ws-external"]