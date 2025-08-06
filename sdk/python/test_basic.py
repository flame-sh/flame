#!/usr/bin/env python3
"""
Basic test for the Flame Python SDK types and enums.
"""

def test_types():
    """Test basic type imports and enum values."""
    try:
        from flame.types import (
            SessionState, TaskState, ApplicationState, Shim, FlameErrorCode,
            SessionAttributes, ApplicationAttributes, FlameError
        )
        
        # Test enum values
        assert SessionState.OPEN == 0
        assert SessionState.CLOSED == 1
        
        assert TaskState.PENDING == 0
        assert TaskState.RUNNING == 1
        assert TaskState.SUCCEED == 2
        assert TaskState.FAILED == 3
        
        assert ApplicationState.ENABLED == 0
        assert ApplicationState.DISABLED == 1
        
        assert Shim.LOG == 0
        assert Shim.STDIO == 1
        assert Shim.WASM == 2
        assert Shim.SHELL == 3
        assert Shim.GRPC == 4
        
        assert FlameErrorCode.INVALID_CONFIG == 0
        assert FlameErrorCode.INVALID_STATE == 1
        assert FlameErrorCode.INVALID_ARGUMENT == 2
        assert FlameErrorCode.INTERNAL == 3
        
        # Test SessionAttributes
        attrs = SessionAttributes(
            application="test-app",
            slots=2,
            common_data=b"test data"
        )
        assert attrs.application == "test-app"
        assert attrs.slots == 2
        assert attrs.common_data == b"test data"
        
        # Test ApplicationAttributes
        app_attrs = ApplicationAttributes(
            name="test-app",
            shim=Shim.SHELL,
            command="python",
            arguments=["script.py"],
            environments=["PATH=/usr/bin"],
            working_directory="/tmp"
        )
        assert app_attrs.name == "test-app"
        assert app_attrs.shim == Shim.SHELL
        assert app_attrs.command == "python"
        assert app_attrs.arguments == ["script.py"]
        assert app_attrs.environments == ["PATH=/usr/bin"]
        assert app_attrs.working_directory == "/tmp"
        
        # Test FlameError
        error = FlameError(FlameErrorCode.INVALID_CONFIG, "Test error")
        assert error.code == FlameErrorCode.INVALID_CONFIG
        assert error.message == "Test error"
        assert "Test error" in str(error)
        
        print("✅ All basic type tests passed!")
        return True
        
    except ImportError as e:
        print(f"❌ Import error: {e}")
        return False
    except Exception as e:
        print(f"❌ Test error: {e}")
        return False


def test_imports():
    """Test that all main imports work."""
    try:
        from flame import (
            SessionState, TaskState, ApplicationState, Shim, FlameErrorCode,
            SessionAttributes, ApplicationAttributes, FlameError, TaskInformer
        )
        print("✅ All imports successful!")
        return True
    except Exception as e:
        print(f"❌ Import error: {e}")
        return False


if __name__ == "__main__":
    print("Testing Flame Python SDK...")
    
    success = True
    success &= test_imports()
    success &= test_types()
    
    if success:
        print("\n🎉 All tests passed! The SDK is working correctly.")
    else:
        print("\n❌ Some tests failed. Please check the errors above.") 