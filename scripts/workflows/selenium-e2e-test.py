#!/usr/bin/env python3
import time

from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.common.desired_capabilities import DesiredCapabilities
import chromedriver_autoinstaller
from pyvirtualdisplay import Display

display = Display(visible=0, size=(800, 800))
display.start()


class TestE2eCandidCanister:
    url_template = "{host}/?canisterId={canister_id}?id={asset_canister_id}"

    def __init__(self, host: str, canister_id: str, asset_canister_id: str):
        self.canister_id = canister_id
        self.asset_canister_id = asset_canister_id
        self.host = host
        self.url = self.url_template.format(
            host=host, canister_id=canister_id, asset_canister_id=asset_canister_id
        )

    def test_candid_ui_canister_interactions(self, driver: webdriver.Chrome):
        driver.get(self.url)

    def test_candid_ui_canister_console_log(self, driver: webdriver.Chrome):
        time.sleep(5)
        driver.get_log("browser")

    def test(self, driver: webdriver.Chrome):
        self.test_candid_ui_canister_interactions(driver)
        self.test_candid_ui_canister_console_log(driver)


class TestE2eFrontendCanister:
    url_template = "{host}/?canisterId={canister_id}"

    def __init__(self, host: str, canister_id: str):
        self.host = host
        self.canister_id = canister_id
        self.url = self.url_template.format(host=host, canister_id=canister_id)

    def test_asset_canister_interactions(self, driver: webdriver.Chrome):
        driver.get(self.url)

    def test_asset_canister_console_log(self, driver: webdriver.Chrome):
        time.sleep(5)
        driver.get_log("browser")

    def test(self, driver: webdriver.Chrome):
        self.test_asset_canister_interactions(driver)
        self.test_asset_canister_console_log(driver)


def main():
    def prepare_driver():
        # Check if the current version of chromedriver exists
        # and if it doesn't exist, download it automatically,
        # then add chromedriver to path
        chromedriver_autoinstaller.install()

        chrome_options = webdriver.ChromeOptions()
        options = [
            # Define window size here
            "--window-size=1200,1200",
            "--ignore-certificate-errors"
            # "--headless",
            # "--disable-gpu",
            # "--window-size=1920,1200",
            # "--ignore-certificate-errors",
            # "--disable-extensions",
            # "--no-sandbox",
            # "--disable-dev-shm-usage",
            #'--remote-debugging-port=9222'
        ]
        for option in options:
            chrome_options.add_argument(option)

        # enable browser logging
        capabilities = webdriver.DesiredCapabilities.CHROME.copy()
        capabilities["goog:loggingPrefs"] = {"browser": "INFO"}
        chrome_options.add_experimental_option("excludeSwitches", ["enable-logging"])
        driver = webdriver.Chrome(
            desired_capabilities=capabilities, options=chrome_options
        )

        return driver

    driver = prepare_driver()

    # TODO: get these from CLI
    host = "http://127.0.0.1:8000"
    candid_canister_id = "r7inp-6aaaa-aaaaa-aaabq-cai"
    frontend_canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai"

    candid_e2e = TestE2eCandidCanister(host, candid_canister_id, frontend_canister_id)
    frontend_e2e = TestE2eFrontendCanister(host, frontend_canister_id)

    candid_e2e.test(driver)
    frontend_e2e.test(driver)


if __name__ == "__main__":
    main()
