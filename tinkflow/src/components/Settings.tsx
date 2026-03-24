export function Settings() {
    return (
        <div className="settings-container">
            <h2>Settings</h2>

            <div className="settings-group">
                <label>
                    Dictation Hotkey:
                    <input type="text" value="Ctrl+Space" readOnly />
                </label>
            </div>

            <div className="settings-group">
                <label>
                    Model Size:
                    <select defaultValue="tiny.en">
                        <option value="tiny.en">Tiny (Fastest, ~75MB)</option>
                        <option value="small.en">Small (Balanced, ~500MB)</option>
                        <option value="medium.en">Medium (Accurate, ~1.5GB)</option>
                    </select>
                </label>
            </div>
        </div>
    );
}
