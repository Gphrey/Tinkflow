!macro NSIS_HOOK_PREUNINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION "Do you also want to delete Tinkflow's locally stored data?$\r$\n$\r$\nThis removes settings, transcription history, personal corrections, and downloaded Whisper models from:$\r$\n$APPDATA\com.admin.tinkflow" IDNO keep_tinkflow_local_data

  RMDir /r "$APPDATA\com.admin.tinkflow"

  keep_tinkflow_local_data:
!macroend
