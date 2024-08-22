export default class TokenStore {
    static STORAGE_TOKEN = "token";
    static STORAGE_USER = "token_owner";

    static setToken(user: string, token: string) {
        localStorage.setItem(TokenStore.STORAGE_TOKEN, token);
        localStorage.setItem(TokenStore.STORAGE_USER, user);
    }

    static getToken(): string {
        return localStorage.getItem(TokenStore.STORAGE_TOKEN) ?? "";
    }

    static getTokenOwner(): string {
        return localStorage.getItem(TokenStore.STORAGE_USER) ?? "";
    }

}