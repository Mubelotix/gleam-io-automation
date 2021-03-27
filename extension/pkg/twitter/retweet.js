eval(`function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function run() {
    try {
        await sleep(7000);
        console.log("running");

        var button = document.evaluate("//div[@data-testid='confirmationSheetConfirm']//div[1]", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
        button.click();

        console.log("done");
        await sleep(3000);
        window.close();
    }
    catch (ex) {
        console.log("failed run");
        await sleep(3000);
        window.close();
    }
}

run()`)