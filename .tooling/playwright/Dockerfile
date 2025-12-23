FROM mcr.microsoft.com/playwright:latest

# Create app dir
WORKDIR /work

# Copy script and install Playwright as a local dependency so require('playwright') works
COPY screenshot.js /work/screenshot.js

# Create package.json and install a Playwright version that matches the base image
ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
RUN npm init -y && \
    npm install --no-audit --no-fund playwright@1.46.1

# Default command just runs the screenshot script
CMD ["node", "screenshot.js"]
