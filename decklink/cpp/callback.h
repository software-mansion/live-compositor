#include "api.h"
#include <cstdint>

class InputCallbackWrapper : public IDeckLinkInputCallback {
private:
  rust::Box<DynInputCallback> cb;
  int32_t refcount = 1;

public:
  InputCallbackWrapper(rust::Box<DynInputCallback> cb) : cb(std::move(cb)){};

  virtual HRESULT STDMETHODCALLTYPE QueryInterface(REFIID, LPVOID *) {
    return E_NOINTERFACE;
  }

  virtual ULONG STDMETHODCALLTYPE AddRef(void);
  virtual ULONG STDMETHODCALLTYPE Release(void);

  virtual HRESULT STDMETHODCALLTYPE
  VideoInputFrameArrived(IDeckLinkVideoInputFrame *video_frame,
                         IDeckLinkAudioInputPacket *audio_packet);
  virtual HRESULT STDMETHODCALLTYPE
  VideoInputFormatChanged(BMDVideoInputFormatChangedEvents events,
                          IDeckLinkDisplayMode *display_mode,
                          BMDDetectedVideoInputFormatFlags flags);
};
