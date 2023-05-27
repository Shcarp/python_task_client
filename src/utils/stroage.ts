
class Stroage {
    private stroage = new Map<string, any>();

    set(key: string, value: any) {
        this.stroage.set(key, value);
    }

    get(key: string) {
        return this.stroage.get(key);
    }
}

export const stroage = new Stroage();
