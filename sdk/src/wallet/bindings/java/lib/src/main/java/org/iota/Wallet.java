// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota;

import com.google.gson.*;
import org.iota.api.WalletCommand;
import org.iota.api.CustomGson;
import org.iota.api.NativeApi;
import org.iota.external.logger.LoggerOutputConfigBuilder;
import org.iota.types.*;
import org.iota.types.addresses.Address;
import org.iota.types.events.Event;
import org.iota.types.events.EventListener;
import org.iota.types.events.transaction.TransactionProgressEvent;
import org.iota.types.events.wallet.WalletEvent;
import org.iota.types.events.wallet.WalletEventType;
import org.iota.types.exceptions.InitializeWalletException;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.ids.account.AccountIndex;
import org.iota.types.account_methods.GenerateAddressOptions;

public class Wallet extends NativeApi {

    /**
     * Initialise the logger used
     * 
     * @param builder the configuration builder of the logger
     */
    public static void initLogger(LoggerOutputConfigBuilder builder) {
        NativeApi.initLogger(CustomGson.get().toJsonTree(builder).toString());
    }

    public Wallet(WalletConfig config) throws InitializeWalletException {
        super(config);
    }

    // Account manager APIs

    /**
     * Create an account with the given alias and return an Account for it.
     *
     * @param alias The name of the account.
     * @return An Account object.
     */
    public Account createAccount(String alias) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        AccountDetails a = CustomGson.get().fromJson(callBaseApi(new WalletCommand("createAccount", o)), AccountDetails.class);
        Account handle = new Account(this, new AccountIndex(a.getIndex()));

        return handle;
    }

    /**
     * Return a given account from the wallet.
     *
     * @param accountIdentifier The account identifier by alias.
     * @return An Account object.
     */
    public Account getAccount(String accountIdentifier) throws WalletException {
        return this.getAccount(new AccountAlias(accountIdentifier));
    }

    /**
     * Return a given account from the wallet.
     *
     * @param accountIdentifier The account identifier.
     * @return An Account object.
     */
    public Account getAccount(AccountIdentifier accountIdentifier) throws WalletException {
        JsonObject o = new JsonObject();
        o.add("accountId", CustomGson.get().toJsonTree(accountIdentifier));

        AccountDetails a = CustomGson.get().fromJson(
                callBaseApi(new WalletCommand("getAccount", o)),
                AccountDetails.class);
        Account handle = new Account(this, new AccountIndex(a.getIndex()));

        return handle;
    }

    /**
     * Returns all the accounts from the wallet.
     *
     * @return An array of Accounts.
     */
    public Account[] getAccounts() throws WalletException {
        JsonArray responsePayload = (JsonArray) callBaseApi(new WalletCommand("getAccounts"));

        Account[] accounts = new Account[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accounts[i] = new Account(this, new AccountIndex(
                    CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), AccountDetails.class).getIndex()));

        return accounts;
    }

    /**
     * Backup the wallet to the specified destination, encrypting it with the
     * specified password.
     *
     * @param destination The path to the file to be created.
     * @param password    The password to encrypt the backup with.
     */
    public void backup(String destination, String password) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("destination", destination);
        o.addProperty("password", password);

        callBaseApi(new WalletCommand("backup", o));
    }

    /**
     * Change the password of the Stronghold file.
     *
     * @param currentPassword The current password for the Stronghold
     * @param newPassword     The new password you want to use for your Stronghold.
     */
    public void changeStrongholdPassword(String currentPassword, String newPassword) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("currentPassword", currentPassword);
        o.addProperty("newPassword", newPassword);

        callBaseApi(new WalletCommand("changeStrongholdPassword", o));
    }

    /**
     * Clears the Stronghold password from memory.
     */
    public void clearStrongholdPassword() throws WalletException {
        callBaseApi(new WalletCommand("clearStrongholdPassword"));
    }

    /**
     * Checks if the Stronghold password is available.
     *
     * @return A boolean value.
     */
    public boolean isStrongholdPasswordAvailable() throws WalletException {
        return callBaseApi(new WalletCommand("isStrongholdPasswordAvailable")).getAsBoolean();
    }

    /**
     * Find accounts with unspent outputs.
     */
    public void recoverAccounts(int accountStartIndex, int accountGapLimit, int addressGapLimit,
            SyncOptions syncOptions) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("accountStartIndex", accountStartIndex);
        o.addProperty("accountGapLimit", accountGapLimit);
        o.addProperty("addressGapLimit", addressGapLimit);
        o.add("syncOptions", CustomGson.get().toJsonTree(syncOptions));

        callBaseApi(new WalletCommand("recoverAccounts", o));
    }

    /**
     * Restore a backup from a Stronghold file
     * Replaces client_options, coin_type, secret_manager and accounts. Returns an
     * error if accounts were already
     * created If Stronghold is used as secret_manager, the existing Stronghold file
     * will be overwritten. If a
     * mnemonic was stored, it will be gone.
     *
     * @param source   The path to the backup file.
     * @param password The password you used to encrypt the backup file.
     */
    public void restoreBackup(String source, String password) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("source", source);
        o.addProperty("password", password);

        callBaseApi(new WalletCommand("restoreBackup", o));
    }

    /**
     * Removes the latest account (account with the largest account index).
     */
    public void removeLatestAccount() throws WalletException {
        callBaseApi(new WalletCommand("removeLatestAccount"));
    }

    /**
     * Generate a mnemonic phrase
     *
     * @return A string of words.
     */
    public String generateMnemonic() throws WalletException {
        return callBaseApi(new WalletCommand("generateMnemonic")).getAsString();
    }

    /**
     * Checks if the given mnemonic is valid.
     *
     * @param mnemonic The mnemonic to verify.
     */
    public void verifyMnemonic(String mnemonic) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("mnemonic", mnemonic);

        callBaseApi(new WalletCommand("verifyMnemonic", o));
    }

    /**
     * Updates the client options for all accounts.
     *
     * @param config A ClientConfig object that contains the options to set.
     */
    public void setClientOptions(ClientConfig config) throws WalletException {
        JsonObject o = new JsonObject();
        // Must use a new Gson instance to not serialize null values.
        // CustomGson.get() would serialize null values and doesn't work here
        o.add("clientOptions", CustomGson.get().toJsonTree(config));

        callBaseApi(new WalletCommand("setClientOptions", o));
    }

    /**
     * Get the status of the Ledger Nano.
     *
     * @return The status of the Ledger Nano
     */
    public LedgerNanoStatus getLedgerNanoStatus() throws WalletException {
        return CustomGson.get().fromJson(callBaseApi(new WalletCommand("getLedgerNanoStatus")), LedgerNanoStatus.class);
    }

    /**
     * Get node information.
     *
     * @param url  The URL of the node you want information from.
     * @param auth The authentication information for the node.
     * @return A JsonObject
     */
    public JsonObject getNodeInfo(String url, NodeAuth auth) throws WalletException {
        JsonObject p = new JsonObject();
        p.addProperty("url", url);
        p.add("auth", CustomGson.get().toJsonTree(auth));

        return (JsonObject) callBaseApi(new WalletCommand("getNodeInfo", p));
    }

    /**
     * Set the stronghold password clear interval.
     *
     * @param password The password to set for the stronghold.
     */
    public void setStrongholdPassword(String password) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("password", password);

        callBaseApi(new WalletCommand("setStrongholdPassword", o));
    }

    /**
     * Set the stronghold password clear interval.
     *
     * @param interval The number of seconds to wait before clearing the password.
     */
    public void setStrongholdPasswordClearInterval(int interval) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("intervalInMilliseconds", interval);

        callBaseApi(new WalletCommand("setStrongholdPasswordClearInterval", o));
    }

    /**
     * Store a mnemonic into the Stronghold vault.
     *
     * @param mnemonic The mnemonic to store.
     */
    public void storeMnemonic(String mnemonic) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("mnemonic", mnemonic);

        callBaseApi(new WalletCommand("storeMnemonic", o));
    }

    /**
     * Start a background sync with the specified options and interval.
     *
     * @param options                The options for the sync.
     * @param intervalInMilliseconds The interval in milliseconds at which the
     *                               background sync will be performed.
     */
    public void startBackgroundSync(SyncOptions options, int intervalInMilliseconds) throws WalletException {
        JsonObject o = new JsonObject();
        o.add("options", CustomGson.get().toJsonTree(options));
        o.addProperty("intervalInMilliseconds", intervalInMilliseconds);

        callBaseApi(new WalletCommand("startBackgroundSync", o));
    }

    /**
     * Stop the background sync process.
     */
    public void stopBackgroundSync() throws WalletException {
        callBaseApi(new WalletCommand("stopBackgroundSync"));
    }

    /**
     * Emits an event for testing if the event system is working
     *
     * @param event The event to emit.
     */
    public void emitTestEvent(WalletEvent event) throws WalletException {
        JsonObject o = new JsonObject();
        o.add("event", CustomGson.get().toJsonTree(event));

        callBaseApi(new WalletCommand("emitTestEvent", o));
    }

    /**
     * Generate an address.
     *
     * @param options The options.
     * @return The generated address.
     */
    public String generateAddress(int accountIndex, int addressIndex,
            GenerateAddressOptions options, String bechHrp) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("accountIndex", accountIndex);
        o.addProperty("addressIndex", addressIndex);
        o.add("options", CustomGson.get().toJsonTree(options));
        o.addProperty("bech32Hrp", bechHrp);

        return callBaseApi(new WalletCommand("generateAddress", o)).getAsString();
    }

    /**
     * Converts a bech32 address to a hex address.
     *
     * @param bech32 The bech32 string to convert to hex.
     * @return A hex string.
     */
    public String bech32ToHex(String bech32) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("bech32Address", bech32);

        return callBaseApi(new WalletCommand("bech32ToHex", o)).getAsString();
    }

    /**
     * Converts a hex address to a bech32 address.
     *
     * @param hex       The hex address to convert.
     * @param bech32Hrp The bech32 human-readable part.
     * @return The bech32 address.
     */
    public String hexToBech32(String hex, String bech32Hrp) throws WalletException {
        JsonObject p = new JsonObject();
        p.addProperty("hex", hex);
        p.addProperty("bech32Hrp", bech32Hrp);

        return callBaseApi(new WalletCommand("hexToBech32", p)).getAsString();
    }

    /**
     * Listen to wallet events, empty vec will listen to all events
     * 
     * @param listener The Listener object hat will handle events
     * @param types    The types you want to listen. Empty means all events
     * @throws WalletException
     */
    public void listen(EventListener listener, WalletEventType... types) throws WalletException {
        callListen(listener, types);
    }

    /**
     * Destroy the Wallet and drop its database connection.
     * Unregisteres any existing listeners.
     */
    public void destroy() throws WalletException {
        clearListeners();
        destroyHandle();
    }

    /**
     * Clear the callbacks for provided events. An null or empty array will clear
     * all listeners.
     *
     * @param types The event types to clear. Empty means clear all events
     */
    public void clearListeners(WalletEventType... types) throws WalletException {
        if (types == null) {
            types = new WalletEventType[0];
        }
        JsonArray p = new JsonArray();
        for (WalletEventType type : types)
            p.add(type.toString());

        JsonObject o = new JsonObject();
        o.add("eventTypes", p);

        callBaseApi(new WalletCommand("clearListeners", o));
    }
}
