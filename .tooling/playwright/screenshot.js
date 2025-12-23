const { chromium } = require('playwright');
const { execSync } = require('child_process');

(async () => {
  const url = process.argv[2] || 'https://pravda.net.ua/';
  const out = process.argv[3] || 'pravda-frontpage.png';

  try {
    // Fetch HTML with curl, ignoring TLS issues. This avoids Playwright's navigation TLS checks.
    console.log('Fetching HTML with curl (ignoring TLS)...');
    const html = execSync(`curl -k -L ${JSON.stringify(url)}`, { encoding: 'utf8', maxBuffer: 10 * 1024 * 1024 });

    // Launch Chromium
    const browser = await chromium.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
    const context = await browser.newContext({ ignoreHTTPSErrors: true });
    const page = await context.newPage();

    // Set the fetched HTML as the page content. Note: external resources (CSS/JS/images)
    // referenced by absolute https URLs may still fail to load if their TLS is invalid. 
    await page.setContent(html, { waitUntil: 'networkidle', timeout: 60000 });

    // Optionally wait a short time for client-side JS to run
    await page.waitForTimeout(2000);

    await page.screenshot({ path: out, fullPage: true });

    await browser.close();
    console.log('Saved', out);
  } catch (err) {
    console.error('Error taking screenshot:', err);
    process.exit(1);
  }
})();
